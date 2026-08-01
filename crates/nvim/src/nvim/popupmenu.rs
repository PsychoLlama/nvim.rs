use crate::src::nvim::api::buffer::nvim_buf_set_lines;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, arena_array, cstr_as_string, cstr_to_string,
};
use crate::src::nvim::api::win_config::parse_winborder;
use crate::src::nvim::autocmd::{block_autocmds, unblock_autocmds};
use crate::src::nvim::buffer::{bt_nofile, buf_clear};
use crate::src::nvim::charset::{ptr2cells, transstr, vim_strsize};
use crate::src::nvim::cmdexpand::{cmdline_compl_is_fuzzy, cmdline_compl_pattern};
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, redraw_later, setcursor_mayforce, update_screen,
};
use crate::src::nvim::eval::typval::{tv_dict_add_bool, tv_dict_add_float, tv_dict_add_nr};
use crate::src::nvim::ex_cmds::{do_ecmd, prepare_tagpreview};
use crate::src::nvim::fuzzy::fuzzy_match_str_with_pos;
use crate::src::nvim::garray::ga_clear;
use crate::src::nvim::getchar::{vgetc, vungetc};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::grid::{
    get_win_by_grid_handle, grid_alloc, grid_assign_handle, grid_draw_border, grid_free,
    grid_invalidate, grid_line_fill, grid_line_flush, grid_line_put_schar, grid_line_puts,
    schar_from_str, screengrid_line_start,
};
use crate::src::nvim::highlight::win_hl_attr;
use crate::src::nvim::highlight::{hl_combine_attr, hl_get_ui_attr};
use crate::src::nvim::highlight_group::{
    HLF_PBR, HLF_PMNI, HLF_PMSI, HLF_PNI, HLF_PNK, HLF_PNX, HLF_PSB, HLF_PSI, HLF_PSK, HLF_PST,
    HLF_PSX, syn_check_group,
};
use crate::src::nvim::insexpand::{
    compl_match_curr_select, get_cot_flags, ins_compl_active, ins_compl_leader,
};
use crate::src::nvim::keycodes::{K_DOWN, K_UP};
use crate::src::nvim::main::{
    Columns, RedrawingDisabled, Rows, State, cia_flags, cmdline_row, cmdline_win, cmdwin_type,
    curbuf, curtab, curwin, default_grid, e_menu_only_exists_in_another_mode, firstwin,
    g_do_tagpreview, hl_attr_active, linebuf_attr, linebuf_char, mouse_col, mouse_grid, mouse_row,
    must_redraw_pum, no_u_sync, p_mousemev, p_pb, p_ph, p_pmw, p_pumborder, p_pvh, p_pw, pum_grid,
    textlock,
};
use crate::src::nvim::mbyte::{mb_string2cells, mb_strnicmp, utf_ptr2cells, utfc_ptr2len};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_finish, arena_mem_free, strequal, xcalloc, xfree, xmalloc, xrealloc, xstrdup,
};
use crate::src::nvim::menu::{execute_menu, get_menu_mode_flag, menu_find, menu_is_separator};
use crate::src::nvim::message::emsg;
use crate::src::nvim::mouse::mouse_find_win_outer;
use crate::src::nvim::r#move::{
    update_topline, validate_cheight, validate_cursor, validate_cursor_col,
};
use crate::src::nvim::option::set_option_value_give_err;
use crate::src::nvim::options::{
    kOptBufhidden, kOptBuflisted, kOptBuftype, kOptCotFlagFuzzy, kOptCotFlagPopup,
    kOptCotFlagPreview, kOptDiff, kOptSwapfile, opt_winborder_values,
};
use crate::src::nvim::os::libc::{__assert_fail, gettext, memset, strchr, strlen};
use crate::src::nvim::plines::plines_m_win;
use crate::src::nvim::plines::win_linetabsize;
use crate::src::nvim::state::MODE_CMDLINE;
use crate::src::nvim::strings::reverse_text;
use crate::src::nvim::types::api::kErrorTypeNone;
use crate::src::nvim::types::ui::{kUICmdline, kUIMultigrid, kUIPopupmenu, kUIWildmenu};
use crate::src::nvim::types::{
    AlignTextPos, Arena, Array, BoolVarValue, Buffer, CMD_index, Error, Float, FloatAnchor,
    FloatRelative, Integer, Object, OptInt, OptVal, OptValData, OptValType, String_0, TriState,
    VirtText, VirtTextChunk, WinConfig, WinSplit, WinStyle, Window, buf_T, cmd_addr_T, colnr_T,
    cstack_T, dict_T, exarg_T, float_T, garray_T, handle_T, hlf_T, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeString, key_extra, linenr_T, lpos_T, object,
    object_data as C2Rust_Unnamed_12, pumitem_T, sattr_T, schar_T, size_t, tabpage_T, uint32_t,
    uint64_t, varnumber_T, vimmenu_T, win_T,
};
use crate::src::nvim::ui::{
    ui_call_grid_destroy, ui_call_grid_resize, ui_call_option_set, ui_call_popupmenu_hide,
    ui_call_popupmenu_select, ui_call_popupmenu_show, ui_call_win_close, ui_call_win_float_pos,
    ui_has, ui_pum_get_height, ui_pum_get_pos,
};
use crate::src::nvim::ui_compositor::{ui_comp_put_grid, ui_comp_remove_grid};
use crate::src::nvim::window::{
    goto_tabpage_tp, valid_tabpage, win_close, win_enter, win_setheight, win_valid,
};
use crate::src::nvim::winfloat::{
    win_config_float, win_float_create_preview, win_float_find_preview,
};
unsafe extern "C" {
    static pum_want: GlobalCell<C2Rust_Unnamed_24>;
}
pub const kFalse: TriState = 0;
pub const kBoolVarTrue: BoolVarValue = 1;
pub const kBoolVarFalse: BoolVarValue = 0;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub const kFloatRelativeEditor: FloatRelative = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const MAXCOL: C2Rust_Unnamed_13 = 2147483647;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kZIndexCmdlinePopupMenu: C2Rust_Unnamed_14 = 250;
pub const kZIndexPopupMenu: C2Rust_Unnamed_14 = 100;
pub const kZIndexFloatDefault: C2Rust_Unnamed_14 = 50;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeBoolean: OptValType = 0;
pub const CMD_append: CMD_index = 0;
pub const ADDR_LINES: cmd_addr_T = 0;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_19 = 1;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CPT_MENU: C2Rust_Unnamed_21 = 2;
pub const CPT_KIND: C2Rust_Unnamed_21 = 1;
pub const CPT_ABBR: C2Rust_Unnamed_21 = 0;
pub const KE_MOUSEMOVE: key_extra = 100;
pub const KE_MOUSEUP: key_extra = 76;
pub const KE_MOUSEDOWN: key_extra = 75;
pub const KE_LEFTMOUSE_NM: key_extra = 69;
pub const KE_RIGHTRELEASE: key_extra = 52;
pub const KE_RIGHTDRAG: key_extra = 51;
pub const KE_RIGHTMOUSE: key_extra = 50;
pub const KE_LEFTDRAG: key_extra = 45;
pub const KE_LEFTMOUSE: key_extra = 44;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const OPT_LOCAL: C2Rust_Unnamed_22 = 2;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_24 {
    pub active: bool,
    pub item: ::core::ffi::c_int,
    pub insert: bool,
    pub finish: bool,
}
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const TAB: ::core::ffi::c_int = '\t' as ::core::ffi::c_int;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CAR: ::core::ffi::c_int = '\r' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const Ctrl_C: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const DEFAULT_GRID_HANDLE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
static pum_array: GlobalCell<*mut pumitem_T> =
    GlobalCell::new(::core::ptr::null_mut::<pumitem_T>());
static pum_size: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_selected: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_first: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static pum_height: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_base_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_kind_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_extra_width: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_scrollbar: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_rl: GlobalCell<bool> = GlobalCell::new(false);
static pum_anchor_grid: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_row: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_win_row_offset: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_win_col_offset: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_left_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_right_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
static pum_above: GlobalCell<bool> = GlobalCell::new(false);
static pum_is_visible: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static pum_is_drawn: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static pum_external: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
static pum_invalid: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
unsafe extern "C" fn pum_compute_size() {
    pum_base_width.set(0 as ::core::ffi::c_int);
    pum_kind_width.set(0 as ::core::ffi::c_int);
    pum_extra_width.set(0 as ::core::ffi::c_int);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < pum_size.get() {
        if !(*(*pum_array.ptr()).offset(i as isize)).pum_text.is_null() {
            let mut w: ::core::ffi::c_int =
                vim_strsize((*(*pum_array.ptr()).offset(i as isize)).pum_text);
            if pum_base_width.get() < w {
                pum_base_width.set(w);
            }
        }
        if !(*(*pum_array.ptr()).offset(i as isize)).pum_kind.is_null() {
            let mut w_0: ::core::ffi::c_int =
                vim_strsize((*(*pum_array.ptr()).offset(i as isize)).pum_kind)
                    + 1 as ::core::ffi::c_int;
            if pum_kind_width.get() < w_0 {
                pum_kind_width.set(w_0);
            }
        }
        if !(*(*pum_array.ptr()).offset(i as isize)).pum_extra.is_null() {
            let mut w_1: ::core::ffi::c_int =
                vim_strsize((*(*pum_array.ptr()).offset(i as isize)).pum_extra)
                    + 1 as ::core::ffi::c_int;
            if pum_extra_width.get() < w_1 {
                pum_extra_width.set(w_1);
            }
        }
        i += 1;
    }
}
unsafe extern "C" fn pum_compute_vertical_placement(
    mut size: ::core::ffi::c_int,
    mut target_win: *mut win_T,
    mut pum_win_row: ::core::ffi::c_int,
    mut above_row: ::core::ffi::c_int,
    mut below_row: ::core::ffi::c_int,
    mut pum_border_size: ::core::ffi::c_int,
) {
    let mut context_lines: ::core::ffi::c_int = 0;
    pum_height.set(if size < 10 as ::core::ffi::c_int {
        size
    } else {
        10 as ::core::ffi::c_int
    });
    if p_ph.get() > 0 as OptInt && pum_height.get() as OptInt > p_ph.get() {
        pum_height.set(p_ph.get() as ::core::ffi::c_int);
    }
    if pum_win_row + 2 as ::core::ffi::c_int + pum_border_size >= below_row - pum_height.get()
        && pum_win_row - above_row > (below_row - above_row) / 2 as ::core::ffi::c_int
    {
        pum_above.set(true_0 != 0);
        if State.get() & MODE_CMDLINE != 0 && target_win.is_null() {
            context_lines = 0 as ::core::ffi::c_int;
        } else {
            context_lines =
                if (2 as ::core::ffi::c_int) < (*target_win).w_wrow - (*target_win).w_cline_row {
                    2 as ::core::ffi::c_int
                } else {
                    (*target_win).w_wrow - (*target_win).w_cline_row
                };
        }
        if pum_win_row >= size + context_lines {
            pum_row.set(pum_win_row - size - context_lines);
            pum_height.set(size);
        } else {
            pum_row.set(0 as ::core::ffi::c_int);
            pum_height.set(pum_win_row - context_lines);
        }
        if p_ph.get() > 0 as OptInt && pum_height.get() as OptInt > p_ph.get() {
            (*pum_row.ptr()) += pum_height.get() - p_ph.get() as ::core::ffi::c_int;
            pum_height.set(p_ph.get() as ::core::ffi::c_int);
        }
        if pum_border_size > 0 as ::core::ffi::c_int
            && pum_border_size + pum_row.get() + pum_height.get() >= pum_win_row
        {
            if pum_row.get() < 2 as ::core::ffi::c_int {
                (*pum_height.ptr()) -= pum_border_size;
            } else {
                (*pum_row.ptr()) -= pum_border_size;
            }
        }
    } else {
        pum_above.set(false_0 != 0);
        if State.get() & MODE_CMDLINE != 0 && target_win.is_null() {
            context_lines = 0 as ::core::ffi::c_int;
        } else {
            validate_cheight(target_win);
            let mut cline_visible_offset: ::core::ffi::c_int =
                (*target_win).w_cline_row + (*target_win).w_cline_height - (*target_win).w_wrow;
            context_lines = if (3 as ::core::ffi::c_int) < cline_visible_offset {
                3 as ::core::ffi::c_int
            } else {
                cline_visible_offset
            };
        }
        pum_row.set(pum_win_row + context_lines);
        pum_height.set(if below_row - pum_row.get() < size {
            below_row - pum_row.get()
        } else {
            size
        });
        if p_ph.get() > 0 as OptInt && pum_height.get() as OptInt > p_ph.get() {
            pum_height.set(p_ph.get() as ::core::ffi::c_int);
        }
        if pum_row.get() + pum_height.get() + pum_border_size >= cmdline_row.get() {
            (*pum_height.ptr()) -= pum_border_size;
        }
    }
    if above_row > 0 as ::core::ffi::c_int
        && pum_row.get() < above_row
        && pum_height.get() > above_row
    {
        pum_row.set(above_row);
        pum_height.set(pum_win_row - above_row);
    }
}
unsafe extern "C" fn set_pum_width_aligned_with_cursor(
    mut width: ::core::ffi::c_int,
    mut available_width: ::core::ffi::c_int,
) -> bool {
    let mut end_padding: bool = true_0 != 0;
    if (width as OptInt) < p_pw.get() {
        width = p_pw.get() as ::core::ffi::c_int;
        end_padding = false_0 != 0;
    }
    if p_pmw.get() > 0 as OptInt && width as OptInt > p_pmw.get() {
        width = p_pmw.get() as ::core::ffi::c_int;
        end_padding = false_0 != 0;
    }
    pum_width.set(
        width
            + (if end_padding as ::core::ffi::c_int != 0 && width as OptInt >= p_pw.get() {
                1 as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
    );
    return available_width >= pum_width.get();
}
unsafe extern "C" fn pum_compute_horizontal_placement(
    mut target_win: *mut win_T,
    mut cursor_col: ::core::ffi::c_int,
    mut border_width: ::core::ffi::c_int,
) {
    let mut max_col: ::core::ffi::c_int = if Columns.get()
        > (if !target_win.is_null() {
            (*target_win).w_wincol + (*target_win).w_view_width
        } else {
            0 as ::core::ffi::c_int
        }) {
        Columns.get()
    } else if !target_win.is_null() {
        (*target_win).w_wincol + (*target_win).w_view_width
    } else {
        0 as ::core::ffi::c_int
    };
    let mut desired_width: ::core::ffi::c_int =
        pum_base_width.get() + pum_kind_width.get() + pum_extra_width.get();
    let mut available_width: ::core::ffi::c_int = 0;
    if pum_rl.get() {
        available_width = cursor_col - pum_scrollbar.get() + 1 as ::core::ffi::c_int - border_width;
    } else {
        available_width = max_col - cursor_col - pum_scrollbar.get() - border_width;
    }
    pum_col.set(cursor_col);
    if set_pum_width_aligned_with_cursor(desired_width, available_width) {
        return;
    }
    if available_width as OptInt > p_pw.get() {
        pum_width.set(available_width);
        return;
    }
    if pum_rl.get() {
        available_width = max_col - pum_scrollbar.get() - border_width;
    } else {
        available_width += cursor_col;
    }
    if available_width as OptInt > p_pw.get() {
        pum_width.set(p_pw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
        if pum_rl.get() {
            pum_col.set(pum_width.get() + pum_scrollbar.get() + border_width);
        } else {
            pum_col.set(max_col - pum_width.get() - pum_scrollbar.get() - border_width);
        }
        return;
    }
    if pum_rl.get() {
        pum_col.set(max_col - 1 as ::core::ffi::c_int);
    } else {
        pum_col.set(0 as ::core::ffi::c_int);
    }
    pum_width.set(max_col - pum_scrollbar.get() - border_width);
}
#[inline]
unsafe extern "C" fn pum_border_width() -> ::core::ffi::c_int {
    if *p_pumborder.get() as ::core::ffi::c_int == NUL
        || strequal(
            p_pumborder.get(),
            (*opt_winborder_values.ptr())[7 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
        ) as ::core::ffi::c_int
            != 0
    {
        return 0 as ::core::ffi::c_int;
    }
    return if strequal(
        p_pumborder.get(),
        (*opt_winborder_values.ptr())[3 as ::core::ffi::c_int as usize]
            as *const ::core::ffi::c_char,
    ) as ::core::ffi::c_int
        != 0
    {
        1 as ::core::ffi::c_int
    } else {
        2 as ::core::ffi::c_int
    };
}
pub unsafe extern "C" fn pum_display(
    mut array: *mut pumitem_T,
    mut size: ::core::ffi::c_int,
    mut selected: ::core::ffi::c_int,
    mut array_changed: bool,
    mut cmd_startcol: ::core::ffi::c_int,
) {
    let mut redo_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pum_win_row: ::core::ffi::c_int = 0;
    let mut cursor_col: ::core::ffi::c_int = 0;
    if !pum_is_visible.get() {
        pum_external.set(
            ui_has(kUIPopupmenu) as ::core::ffi::c_int != 0
                || State.get() & MODE_CMDLINE != 0
                    && ui_has(kUIWildmenu) as ::core::ffi::c_int != 0,
        );
    }
    pum_rl.set(
        State.get() & MODE_CMDLINE == 0 as ::core::ffi::c_int
            && (*curwin.get()).w_onebuf_opt.wo_rl != 0,
    );
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    loop {
        pum_is_visible.set(true_0 != 0);
        pum_is_drawn.set(true_0 != 0);
        validate_cursor_col(curwin.get());
        let mut above_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut below_row: ::core::ffi::c_int =
            if cmdline_row.get() > (*curwin.get()).w_winrow + (*curwin.get()).w_view_height {
                cmdline_row.get()
            } else {
                (*curwin.get()).w_winrow + (*curwin.get()).w_view_height
            };
        if State.get() & MODE_CMDLINE != 0 {
            below_row = cmdline_row.get();
        }
        let mut target_win: *mut win_T = if State.get() & MODE_CMDLINE != 0 {
            cmdline_win.get()
        } else {
            curwin.get()
        };
        pum_win_row_offset.set(0 as ::core::ffi::c_int);
        pum_win_col_offset.set(0 as ::core::ffi::c_int);
        if State.get() & MODE_CMDLINE != 0 {
            pum_win_row = if !(*cmdline_win.ptr()).is_null() {
                (*cmdline_win.get()).w_wrow
            } else if ui_has(kUICmdline) as ::core::ffi::c_int != 0 {
                0 as ::core::ffi::c_int
            } else {
                cmdline_row.get()
            };
            cursor_col = (if !(*cmdline_win.ptr()).is_null() {
                (*cmdline_win.get()).w_config._cmdline_offset
            } else {
                0 as ::core::ffi::c_int
            }) + cmd_startcol;
            cursor_col %= if !(*cmdline_win.ptr()).is_null() {
                (*cmdline_win.get()).w_view_width
            } else {
                Columns.get()
            };
            pum_anchor_grid.set(if ui_has(kUICmdline) as ::core::ffi::c_int != 0 {
                -1 as ::core::ffi::c_int
            } else {
                DEFAULT_GRID_HANDLE
            });
        } else {
            pum_win_row = (*curwin.get()).w_wrow;
            if pum_rl.get() {
                cursor_col =
                    (*curwin.get()).w_view_width - (*curwin.get()).w_wcol - 1 as ::core::ffi::c_int;
            } else {
                cursor_col = (*curwin.get()).w_wcol;
            }
        }
        if !target_win.is_null() {
            pum_anchor_grid.set((*(*target_win).w_grid.target).handle as ::core::ffi::c_int);
            pum_win_row += (*target_win).w_grid.row_offset;
            cursor_col += (*target_win).w_grid.col_offset;
            if (*target_win).w_grid.target != default_grid.ptr() {
                pum_win_row += (*target_win).w_winrow;
                cursor_col += (*target_win).w_wincol;
                if !ui_has(kUIMultigrid) {
                    pum_anchor_grid.set(DEFAULT_GRID_HANDLE);
                } else {
                    pum_win_row_offset.set((*target_win).w_winrow);
                    pum_win_col_offset.set((*target_win).w_wincol);
                }
            }
        }
        if pum_external.get() {
            if array_changed {
                let mut arena: Arena = ARENA_EMPTY;
                let mut arr: Array = arena_array(&raw mut arena, size as size_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < size {
                    let mut item: Array = arena_array(&raw mut arena, 4 as size_t);
                    let c2rust_fresh0 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh0 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_text),
                        },
                    };
                    let c2rust_fresh1 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh1 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_kind),
                        },
                    };
                    let c2rust_fresh2 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh2 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_extra),
                        },
                    };
                    let c2rust_fresh3 = item.size;
                    item.size = item.size.wrapping_add(1);
                    *item.items.offset(c2rust_fresh3 as isize) = object {
                        type_0: kObjectTypeString,
                        data: C2Rust_Unnamed_12 {
                            string: cstr_as_string((*array.offset(i as isize)).pum_info),
                        },
                    };
                    let c2rust_fresh4 = arr.size;
                    arr.size = arr.size.wrapping_add(1);
                    *arr.items.offset(c2rust_fresh4 as isize) = object {
                        type_0: kObjectTypeArray,
                        data: C2Rust_Unnamed_12 { array: item },
                    };
                    i += 1;
                }
                ui_call_popupmenu_show(
                    arr,
                    selected as Integer,
                    (pum_win_row - pum_win_row_offset.get()) as Integer,
                    (cursor_col - pum_win_col_offset.get()) as Integer,
                    pum_anchor_grid.get() as Integer,
                );
                arena_mem_free(arena_finish(&raw mut arena));
            } else {
                ui_call_popupmenu_select(selected as Integer);
                return;
            }
        }
        let mut pvwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_onebuf_opt.wo_pvw != 0 {
                pvwin = wp;
                break;
            } else {
                wp = (*wp).w_next;
            }
        }
        if !pvwin.is_null() {
            if (*pvwin).w_winrow < (*curwin.get()).w_winrow {
                above_row = (*pvwin).w_winrow + (*pvwin).w_height;
            } else if (*pvwin).w_winrow > (*curwin.get()).w_winrow + (*curwin.get()).w_height {
                below_row = (*pvwin).w_winrow;
            }
        }
        pum_compute_vertical_placement(
            size,
            target_win,
            pum_win_row,
            above_row,
            below_row,
            border_width,
        );
        if border_width == 0 as ::core::ffi::c_int
            && (pum_height.get() < 1 as ::core::ffi::c_int
                || pum_height.get() == 1 as ::core::ffi::c_int && size > 1 as ::core::ffi::c_int)
        {
            return;
        }
        pum_array.set(array);
        pum_size.set(size);
        if pum_external.get() {
            return;
        }
        pum_compute_size();
        pum_scrollbar.set(if pum_height.get() < size {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
        pum_compute_horizontal_placement(target_win, cursor_col, border_width);
        if !(pum_set_selected(selected, redo_count) as ::core::ffi::c_int != 0 && {
            redo_count += 1;
            redo_count <= 2 as ::core::ffi::c_int
        }) {
            break;
        }
    }
    (*pum_grid.ptr()).zindex = if State.get() & MODE_CMDLINE != 0 {
        kZIndexCmdlinePopupMenu as ::core::ffi::c_int
    } else {
        kZIndexPopupMenu as ::core::ffi::c_int
    };
    pum_redraw();
}
unsafe extern "C" fn pum_compute_text_attrs(
    mut text: *mut ::core::ffi::c_char,
    mut hlf: hlf_T,
    mut user_hlattr: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_int {
    if *text as ::core::ffi::c_int == NUL
        || hlf as ::core::ffi::c_uint != HLF_PSI as ::core::ffi::c_uint
            && hlf as ::core::ffi::c_uint != HLF_PNI as ::core::ffi::c_uint
        || win_hl_attr(curwin.get(), HLF_PMSI) == win_hl_attr(curwin.get(), HLF_PSI)
            && win_hl_attr(curwin.get(), HLF_PMNI) == win_hl_attr(curwin.get(), HLF_PNI)
    {
        return ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
    let mut leader: *mut ::core::ffi::c_char = if State.get() & MODE_CMDLINE != 0 {
        cmdline_compl_pattern()
    } else {
        ins_compl_leader()
    };
    if leader.is_null() || *leader as ::core::ffi::c_int == NUL {
        return ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
    let mut attrs: *mut ::core::ffi::c_int = xmalloc(
        ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(vim_strsize(text) as size_t),
    ) as *mut ::core::ffi::c_int;
    let mut in_fuzzy: bool = if State.get() & MODE_CMDLINE != 0 {
        cmdline_compl_is_fuzzy() as ::core::ffi::c_int
    } else {
        (get_cot_flags() & kOptCotFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint) as ::core::ffi::c_int
    } != 0;
    let mut leader_len: size_t = strlen(leader);
    let mut ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut matched_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if in_fuzzy {
        ga = fuzzy_match_str_with_pos(text, leader);
        if ga.is_null() {
            xfree(attrs as *mut ::core::ffi::c_void);
            return ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
    }
    let mut ptr: *const ::core::ffi::c_char = text;
    let mut cell_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut char_pos: uint32_t = 0 as uint32_t;
    let mut is_select: bool = hlf as ::core::ffi::c_uint == HLF_PSI as ::core::ffi::c_uint;
    while *ptr as ::core::ffi::c_int != NUL {
        let mut new_attr: ::core::ffi::c_int = win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int);
        if !ga.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*ga).ga_len {
                if char_pos == *((*ga).ga_data as *mut uint32_t).offset(i as isize) {
                    new_attr = win_hl_attr(
                        curwin.get(),
                        if is_select as ::core::ffi::c_int != 0 {
                            HLF_PMSI
                        } else {
                            HLF_PMNI
                        },
                    );
                    new_attr = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PMNI), new_attr);
                    new_attr = hl_combine_attr(
                        win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int),
                        new_attr,
                    );
                    break;
                } else {
                    i += 1;
                }
            }
        } else {
            if matched_len < 0 as ::core::ffi::c_int
                && mb_strnicmp(ptr, leader, leader_len) == 0 as ::core::ffi::c_int
            {
                matched_len = leader_len as ::core::ffi::c_int;
            }
            if matched_len > 0 as ::core::ffi::c_int {
                new_attr = win_hl_attr(
                    curwin.get(),
                    if is_select as ::core::ffi::c_int != 0 {
                        HLF_PMSI
                    } else {
                        HLF_PMNI
                    },
                );
                new_attr = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PMNI), new_attr);
                new_attr = hl_combine_attr(
                    win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int),
                    new_attr,
                );
                matched_len -= 1;
            }
        }
        new_attr = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PNI), new_attr);
        if user_hlattr > 0 as ::core::ffi::c_int {
            new_attr = hl_combine_attr(new_attr, user_hlattr);
        }
        let mut char_cells: ::core::ffi::c_int = utf_ptr2cells(ptr);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < char_cells {
            *attrs.offset((cell_idx + i_0) as isize) = new_attr;
            i_0 += 1;
        }
        cell_idx += char_cells;
        ptr = ptr.offset(utfc_ptr2len(ptr as *mut ::core::ffi::c_char) as isize);
        char_pos = char_pos.wrapping_add(1);
    }
    if !ga.is_null() {
        ga_clear(ga);
        xfree(ga as *mut ::core::ffi::c_void);
    }
    return attrs;
}
unsafe extern "C" fn pum_grid_puts_with_attrs(
    mut col: ::core::ffi::c_int,
    mut cells: ::core::ffi::c_int,
    mut text: *const ::core::ffi::c_char,
    mut textlen: ::core::ffi::c_int,
    mut attrs: *const ::core::ffi::c_int,
) {
    let col_start: ::core::ffi::c_int = col;
    let mut ptr: *const ::core::ffi::c_char = text;
    while *ptr as ::core::ffi::c_int != NUL
        && (textlen < 0 as ::core::ffi::c_int || ptr < text.offset(textlen as isize))
    {
        let mut char_len: ::core::ffi::c_int = utfc_ptr2len(ptr);
        let mut attr: ::core::ffi::c_int = *attrs.offset(
            (if pum_rl.get() as ::core::ffi::c_int != 0 {
                col_start + cells - col - 1 as ::core::ffi::c_int
            } else {
                col - col_start
            }) as isize,
        );
        grid_line_puts(col, ptr, char_len, attr);
        col += utf_ptr2cells(ptr);
        ptr = ptr.offset(char_len as isize);
    }
}
#[inline]
unsafe extern "C" fn pum_align_order(mut order: *mut ::core::ffi::c_int) {
    let mut is_default: bool = cia_flags.get() == 0 as ::core::ffi::c_uint;
    *order.offset(0 as ::core::ffi::c_int as isize) = (if is_default as ::core::ffi::c_int != 0 {
        CPT_ABBR as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        (*cia_flags.ptr()).wrapping_div(100 as ::core::ffi::c_uint)
    }) as ::core::ffi::c_int;
    *order.offset(1 as ::core::ffi::c_int as isize) = (if is_default as ::core::ffi::c_int != 0 {
        CPT_KIND as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        (*cia_flags.ptr())
            .wrapping_div(10 as ::core::ffi::c_uint)
            .wrapping_rem(10 as ::core::ffi::c_uint)
    }) as ::core::ffi::c_int;
    *order.offset(2 as ::core::ffi::c_int as isize) = (if is_default as ::core::ffi::c_int != 0 {
        CPT_MENU as ::core::ffi::c_int as ::core::ffi::c_uint
    } else {
        (*cia_flags.ptr()).wrapping_rem(10 as ::core::ffi::c_uint)
    }) as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn pum_get_item(
    mut index: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    match type_0 {
        0 => return (*(*pum_array.ptr()).offset(index as isize)).pum_text,
        1 => return (*(*pum_array.ptr()).offset(index as isize)).pum_kind,
        2 => return (*(*pum_array.ptr()).offset(index as isize)).pum_extra,
        _ => {}
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
#[inline]
unsafe extern "C" fn pum_user_attr_combine(
    mut idx: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut user_attr: [::core::ffi::c_int; 2] = [
        (*(*pum_array.ptr()).offset(idx as isize)).pum_user_abbr_hlattr,
        (*(*pum_array.ptr()).offset(idx as isize)).pum_user_kind_hlattr,
    ];
    return if user_attr[type_0 as usize] > 0 as ::core::ffi::c_int {
        hl_combine_attr(attr, user_attr[type_0 as usize])
    } else {
        attr
    };
}
pub unsafe extern "C" fn pum_redraw() {
    let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut attr_scroll: ::core::ffi::c_int = win_hl_attr(curwin.get(), HLF_PSB);
    let mut attr_thumb: ::core::ffi::c_int = win_hl_attr(curwin.get(), HLF_PST);
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut thumb_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut thumb_height: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    let fcs_trunc: schar_T = if pum_rl.get() as ::core::ffi::c_int != 0 {
        (*curwin.get()).w_p_fcs_chars.truncrl
    } else {
        (*curwin.get()).w_p_fcs_chars.trunc
    };
    let hlfsNorm: [hlf_T; 3] = [HLF_PNI, HLF_PNK, HLF_PNX];
    let hlfsSel: [hlf_T; 3] = [HLF_PSI, HLF_PSK, HLF_PSX];
    let mut grid_width: ::core::ffi::c_int = pum_width.get();
    let mut col_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut extra_space: bool = false_0 != 0;
    if pum_rl.get() {
        col_off = pum_width.get() - 1 as ::core::ffi::c_int;
        '_c2rust_label: {
            if State.get() & MODE_CMDLINE == 0 {
            } else {
                __assert_fail(
                    b"!(State & MODE_CMDLINE)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/popupmenu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    584 as ::core::ffi::c_uint,
                    b"void pum_redraw(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut win_end_col: ::core::ffi::c_int =
            (*curwin.get()).w_wincol + (*curwin.get()).w_width;
        if pum_col.get() < win_end_col - 1 as ::core::ffi::c_int {
            grid_width += 1 as ::core::ffi::c_int;
            extra_space = true_0 != 0;
        }
    } else {
        let mut min_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if pum_col.get() > min_col {
            grid_width += 1 as ::core::ffi::c_int;
            col_off = 1 as ::core::ffi::c_int;
            extra_space = true_0 != 0;
        }
    }
    let mut fconfig: WinConfig = WinConfig {
        window: 0,
        bufpos: lpos_T {
            lnum: -1 as linenr_T,
            col: 0 as colnr_T,
        },
        height: 0 as ::core::ffi::c_int,
        width: 0 as ::core::ffi::c_int,
        row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
        anchor: 0 as FloatAnchor,
        relative: kFloatRelativeEditor,
        external: false_0 != 0,
        focusable: true_0 != 0,
        mouse: true_0 != 0,
        split: kWinSplitLeft,
        zindex: kZIndexFloatDefault as ::core::ffi::c_int,
        style: kWinStyleUnused,
        border: false,
        shadow: false,
        border_chars: [[0; 32]; 8],
        border_hl_ids: [0; 8],
        border_attr: [0; 8],
        title: false,
        title_pos: kAlignLeft,
        title_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        title_width: 0,
        footer: false,
        footer_pos: kAlignLeft,
        footer_chunks: VirtText {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<VirtTextChunk>(),
        },
        footer_width: 0,
        noautocmd: false_0 != 0,
        fixed: false_0 != 0,
        hide: false_0 != 0,
        _cmdline_offset: INT_MAX,
    };
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    let mut border_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut border_char: schar_T = 0 as schar_T;
    let mut fill_char: schar_T = ' ' as ::core::ffi::c_int as schar_T;
    let mut has_border: bool = border_width > 0 as ::core::ffi::c_int;
    if border_width > 0 as ::core::ffi::c_int {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        if !parse_winborder(&raw mut fconfig, p_pumborder.get(), &raw mut err) {
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                emsg(err.msg);
            }
            api_clear_error(&raw mut err);
            return;
        }
        if strequal(
            p_pumborder.get(),
            (*opt_winborder_values.ptr())[3 as ::core::ffi::c_int as usize]
                as *const ::core::ffi::c_char,
        ) {
            fconfig.shadow = true_0 != 0;
            let mut blend: ::core::ffi::c_int = syn_check_group(
                b"PmenuShadow\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
            );
            let mut through: ::core::ffi::c_int = syn_check_group(
                b"PmenuShadowThrough\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
            );
            fconfig.border_hl_ids[2 as ::core::ffi::c_int as usize] = through;
            fconfig.border_hl_ids[3 as ::core::ffi::c_int as usize] = blend;
            fconfig.border_hl_ids[4 as ::core::ffi::c_int as usize] = blend;
            fconfig.border_hl_ids[5 as ::core::ffi::c_int as usize] = blend;
            fconfig.border_hl_ids[6 as ::core::ffi::c_int as usize] = through;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < 8 as ::core::ffi::c_int {
            let mut attr: ::core::ffi::c_int = *(*hl_attr_active.ptr()).offset(HLF_PBR as isize);
            if fconfig.border_hl_ids[i as usize] != 0 {
                attr = hl_get_ui_attr(
                    -1 as ::core::ffi::c_int,
                    HLF_PBR,
                    fconfig.border_hl_ids[i as usize],
                    false_0 != 0,
                );
            }
            fconfig.border_attr[i as usize] = attr;
            i += 1;
        }
        api_clear_error(&raw mut err);
        if pum_scrollbar.get() != 0 {
            border_char = schar_from_str(
                &raw mut *(&raw mut fconfig.border_chars as *mut [::core::ffi::c_char; 32])
                    .offset(3 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_char,
            );
            border_attr = fconfig.border_attr[3 as ::core::ffi::c_int as usize];
        }
    }
    if pum_scrollbar.get() > 0 as ::core::ffi::c_int
        && (!fconfig.border || fconfig.shadow as ::core::ffi::c_int != 0)
    {
        grid_width += 1;
        if pum_rl.get() {
            col_off += 1;
        }
    }
    (*pum_grid.ptr()).blending =
        p_pb.get() > 0 as OptInt || fconfig.shadow as ::core::ffi::c_int != 0;
    grid_assign_handle(pum_grid.ptr());
    pum_left_col.set(pum_col.get() - col_off);
    pum_right_col.set(pum_left_col.get() + grid_width);
    let mut moved: bool = ui_comp_put_grid(
        pum_grid.ptr(),
        pum_row.get(),
        pum_left_col.get(),
        pum_height.get() + border_width,
        grid_width + border_width,
        false_0 != 0,
        true_0 != 0,
    );
    let mut invalid_grid: bool =
        moved as ::core::ffi::c_int != 0 || pum_invalid.get() as ::core::ffi::c_int != 0;
    pum_invalid.set(false_0 != 0);
    must_redraw_pum.set(false_0 != 0);
    if (*pum_grid.ptr()).chars.is_null()
        || (*pum_grid.ptr()).rows != pum_height.get() + border_width
        || (*pum_grid.ptr()).cols != grid_width + border_width
    {
        grid_alloc(
            pum_grid.ptr(),
            pum_height.get() + border_width,
            grid_width + border_width,
            !invalid_grid,
            false_0 != 0,
        );
        ui_call_grid_resize(
            (*pum_grid.ptr()).handle as Integer,
            (*pum_grid.ptr()).cols as Integer,
            (*pum_grid.ptr()).rows as Integer,
        );
    } else if invalid_grid {
        grid_invalidate(pum_grid.ptr());
    }
    if ui_has(kUIMultigrid) {
        let mut anchor: *const ::core::ffi::c_char = if pum_above.get() as ::core::ffi::c_int != 0 {
            b"SW\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NW\0".as_ptr() as *const ::core::ffi::c_char
        };
        let mut row_off: ::core::ffi::c_int = if pum_above.get() as ::core::ffi::c_int != 0 {
            -pum_height.get()
        } else {
            0 as ::core::ffi::c_int
        };
        ui_call_win_float_pos(
            (*pum_grid.ptr()).handle as Integer,
            -1 as Window,
            cstr_as_string(anchor),
            pum_anchor_grid.get() as Integer,
            (pum_row.get() - row_off - pum_win_row_offset.get()) as Float,
            (pum_left_col.get() - pum_win_col_offset.get()) as Float,
            false_0 != 0,
            (*pum_grid.ptr()).zindex as Integer,
            (*pum_grid.ptr()).comp_index as ::core::ffi::c_int as Integer,
            (*pum_grid.ptr()).comp_row as Integer,
            (*pum_grid.ptr()).comp_col as Integer,
        );
    }
    let mut scroll_range: ::core::ffi::c_int = pum_size.get() - pum_height.get();
    if fconfig.border {
        grid_draw_border(
            pum_grid.ptr(),
            &raw mut fconfig,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        if !fconfig.shadow {
            row += 1;
            col_off += 1;
        }
    }
    pum_first.set(if pum_first.get() < scroll_range {
        pum_first.get()
    } else {
        scroll_range
    });
    if pum_scrollbar.get() != 0 {
        thumb_height = pum_height.get() * pum_height.get() / pum_size.get();
        if thumb_height == 0 as ::core::ffi::c_int {
            thumb_height = 1 as ::core::ffi::c_int;
        }
        thumb_pos = (pum_first.get() * (pum_height.get() - thumb_height)
            + scroll_range / 2 as ::core::ffi::c_int)
            / scroll_range;
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < pum_height.get() {
        let mut idx: ::core::ffi::c_int = i_0 + pum_first.get();
        let selected: bool = idx == pum_selected.get();
        let hlfs: *const hlf_T = if selected as ::core::ffi::c_int != 0 {
            &raw const hlfsSel as *const hlf_T
        } else {
            &raw const hlfsNorm as *const hlf_T
        };
        let trunc_attr: ::core::ffi::c_int = win_hl_attr(
            curwin.get(),
            if selected as ::core::ffi::c_int != 0 {
                HLF_PSI
            } else {
                HLF_PNI
            },
        );
        let mut hlf: hlf_T = *hlfs.offset(0 as ::core::ffi::c_int as isize);
        let mut attr_0: ::core::ffi::c_int = win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int);
        attr_0 = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PNI), attr_0);
        screengrid_line_start(pum_grid.ptr(), row, 0 as ::core::ffi::c_int);
        if extra_space {
            if pum_rl.get() {
                grid_line_puts(
                    col_off + 1 as ::core::ffi::c_int,
                    b" \0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    attr_0,
                );
            } else {
                grid_line_puts(
                    col_off - 1 as ::core::ffi::c_int,
                    b" \0".as_ptr() as *const ::core::ffi::c_char,
                    1 as ::core::ffi::c_int,
                    attr_0,
                );
            }
        }
        let mut grid_col: ::core::ffi::c_int = col_off;
        let mut totwidth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut need_fcs_trunc: bool = false_0 != 0;
        let mut order: [::core::ffi::c_int; 3] = [0; 3];
        let mut items_width_array: [::core::ffi::c_int; 3] = [
            pum_base_width.get(),
            pum_kind_width.get(),
            pum_extra_width.get(),
        ];
        pum_align_order(&raw mut order as *mut ::core::ffi::c_int);
        let mut basic_width: ::core::ffi::c_int =
            items_width_array[order[0 as ::core::ffi::c_int as usize] as usize];
        let mut last_isabbr: bool =
            order[2 as ::core::ffi::c_int as usize] == CPT_ABBR as ::core::ffi::c_int;
        let mut orig_attr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < 3 as ::core::ffi::c_int {
            let mut item_type: ::core::ffi::c_int = order[j as usize];
            hlf = *hlfs.offset(item_type as isize);
            attr_0 = win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int);
            attr_0 = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PNI), attr_0);
            orig_attr = attr_0;
            if item_type < 2 as ::core::ffi::c_int {
                attr_0 = pum_user_attr_combine(idx, item_type, attr_0);
            }
            let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            p = pum_get_item(idx, item_type);
            let next_isempty: bool = j + 1 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int
                || pum_get_item(idx, order[(j + 1 as ::core::ffi::c_int) as usize]).is_null();
            if !p.is_null() {
                loop {
                    if s.is_null() {
                        s = p;
                    }
                    let mut w: ::core::ffi::c_int = ptr2cells(p);
                    if *p as ::core::ffi::c_int != NUL
                        && *p as ::core::ffi::c_int != TAB
                        && totwidth + w <= pum_width.get()
                    {
                        width += w;
                    } else {
                        let width_limit: ::core::ffi::c_int = pum_width.get();
                        let mut saved: ::core::ffi::c_char = *p;
                        if saved as ::core::ffi::c_int != NUL {
                            *p = NUL as ::core::ffi::c_char;
                        }
                        let mut st: *mut ::core::ffi::c_char = transstr(s, true_0 != 0);
                        if saved as ::core::ffi::c_int != NUL {
                            *p = saved;
                        }
                        let mut attrs: *mut ::core::ffi::c_int =
                            ::core::ptr::null_mut::<::core::ffi::c_int>();
                        if item_type == CPT_ABBR as ::core::ffi::c_int {
                            attrs = pum_compute_text_attrs(
                                st,
                                hlf,
                                (*(*pum_array.ptr()).offset(idx as isize)).pum_user_abbr_hlattr,
                            );
                        }
                        if pum_rl.get() {
                            let mut rt: *mut ::core::ffi::c_char = reverse_text(st);
                            let mut rt_start: *mut ::core::ffi::c_char = rt;
                            let mut cells: ::core::ffi::c_int =
                                mb_string2cells(rt) as ::core::ffi::c_int;
                            let mut pad: ::core::ffi::c_int =
                                if next_isempty as ::core::ffi::c_int != 0 {
                                    0 as ::core::ffi::c_int
                                } else {
                                    2 as ::core::ffi::c_int
                                };
                            if width_limit - totwidth < cells + pad {
                                need_fcs_trunc = true_0 != 0;
                            }
                            if grid_col - cells < col_off - width_limit {
                                loop {
                                    cells -= utf_ptr2cells(rt);
                                    rt = rt.offset(utfc_ptr2len(rt) as isize);
                                    if grid_col - cells >= col_off - width_limit {
                                        break;
                                    }
                                }
                                if grid_col - cells > col_off - width_limit {
                                    rt = rt.offset(-1);
                                    *rt = '<' as ::core::ffi::c_char;
                                    cells += 1;
                                }
                            }
                            if attrs.is_null() {
                                grid_line_puts(
                                    grid_col - cells + 1 as ::core::ffi::c_int,
                                    rt,
                                    -1 as ::core::ffi::c_int,
                                    attr_0,
                                );
                            } else {
                                pum_grid_puts_with_attrs(
                                    grid_col - cells + 1 as ::core::ffi::c_int,
                                    cells,
                                    rt,
                                    -1 as ::core::ffi::c_int,
                                    attrs,
                                );
                            }
                            xfree(rt_start as *mut ::core::ffi::c_void);
                            xfree(st as *mut ::core::ffi::c_void);
                            grid_col -= width;
                        } else {
                            let mut cells_0: ::core::ffi::c_int =
                                mb_string2cells(st) as ::core::ffi::c_int;
                            let mut pad_0: ::core::ffi::c_int =
                                if next_isempty as ::core::ffi::c_int != 0 {
                                    0 as ::core::ffi::c_int
                                } else {
                                    2 as ::core::ffi::c_int
                                };
                            if width_limit - totwidth < cells_0 + pad_0 {
                                need_fcs_trunc = true_0 != 0;
                            }
                            if need_fcs_trunc {
                                let mut available_cells: ::core::ffi::c_int =
                                    width_limit - totwidth;
                                let mut p_end: *mut ::core::ffi::c_char = st;
                                let mut displayed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                while *p_end as ::core::ffi::c_int != NUL {
                                    let mut char_cells: ::core::ffi::c_int = utf_ptr2cells(p_end);
                                    if displayed + char_cells > available_cells {
                                        break;
                                    }
                                    displayed += char_cells;
                                    p_end = p_end.offset(utfc_ptr2len(p_end) as isize);
                                }
                                *p_end = NUL as ::core::ffi::c_char;
                                cells_0 = displayed;
                                width = displayed;
                            }
                            if attrs.is_null() {
                                grid_line_puts(grid_col, st, -1 as ::core::ffi::c_int, attr_0);
                            } else {
                                pum_grid_puts_with_attrs(
                                    grid_col,
                                    cells_0,
                                    st,
                                    -1 as ::core::ffi::c_int,
                                    attrs,
                                );
                            }
                            xfree(st as *mut ::core::ffi::c_void);
                            grid_col += width;
                        }
                        if !attrs.is_null() {
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut attrs as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            let _ = *ptr_;
                        }
                        if *p as ::core::ffi::c_int != TAB {
                            break;
                        }
                        if pum_rl.get() {
                            grid_line_puts(
                                grid_col - 1 as ::core::ffi::c_int,
                                b"  \0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                                attr_0,
                            );
                            grid_col -= 2 as ::core::ffi::c_int;
                        } else {
                            grid_line_puts(
                                grid_col,
                                b"  \0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                                attr_0,
                            );
                            grid_col += 2 as ::core::ffi::c_int;
                        }
                        totwidth += 2 as ::core::ffi::c_int;
                        s = ::core::ptr::null_mut::<::core::ffi::c_char>();
                        width = 0 as ::core::ffi::c_int;
                    }
                    p = p.offset(utfc_ptr2len(p) as isize);
                }
            }
            if j > 0 as ::core::ffi::c_int {
                n = items_width_array[order[1 as ::core::ffi::c_int as usize] as usize]
                    + (if last_isabbr as ::core::ffi::c_int != 0 {
                        0 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    });
            } else {
                n = if order[j as usize] == CPT_ABBR as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            if j == 2 as ::core::ffi::c_int
                || next_isempty as ::core::ffi::c_int != 0
                    && (j == 1 as ::core::ffi::c_int
                        || j == 0 as ::core::ffi::c_int
                            && pum_get_item(idx, order[(j + 2 as ::core::ffi::c_int) as usize])
                                .is_null())
                || basic_width + n >= pum_width.get()
            {
                break;
            }
            if pum_rl.get() {
                grid_line_fill(
                    col_off - basic_width - n + 1 as ::core::ffi::c_int,
                    grid_col + 1 as ::core::ffi::c_int,
                    ' ' as ::core::ffi::c_int as schar_T,
                    orig_attr,
                );
                grid_col = col_off - basic_width - n;
            } else {
                grid_line_fill(
                    grid_col,
                    col_off + basic_width + n,
                    ' ' as ::core::ffi::c_int as schar_T,
                    orig_attr,
                );
                grid_col = col_off + basic_width + n;
            }
            totwidth = basic_width + n;
            j += 1;
        }
        if pum_rl.get() {
            let lcol: ::core::ffi::c_int = col_off - pum_width.get() + 1 as ::core::ffi::c_int;
            grid_line_fill(
                lcol,
                grid_col + 1 as ::core::ffi::c_int,
                ' ' as ::core::ffi::c_int as schar_T,
                orig_attr,
            );
            if need_fcs_trunc {
                *(*linebuf_char.ptr()).offset(lcol as isize) = if fcs_trunc != NUL as schar_T {
                    fcs_trunc
                } else {
                    '<' as ::core::ffi::c_int as schar_T
                };
                *(*linebuf_attr.ptr()).offset(lcol as isize) = trunc_attr as sattr_T;
                if pum_width.get() > 1 as ::core::ffi::c_int
                    && *(*linebuf_char.ptr()).offset((lcol + 1 as ::core::ffi::c_int) as isize)
                        == NUL as schar_T
                {
                    *(*linebuf_char.ptr()).offset((lcol + 1 as ::core::ffi::c_int) as isize) =
                        ' ' as ::core::ffi::c_int as schar_T;
                }
            }
        } else {
            let rcol: ::core::ffi::c_int = col_off + pum_width.get();
            grid_line_fill(
                grid_col,
                rcol,
                ' ' as ::core::ffi::c_int as schar_T,
                orig_attr,
            );
            if need_fcs_trunc {
                if pum_width.get() > 1 as ::core::ffi::c_int
                    && *(*linebuf_char.ptr()).offset((rcol - 1 as ::core::ffi::c_int) as isize)
                        == NUL as schar_T
                {
                    *(*linebuf_char.ptr()).offset((rcol - 2 as ::core::ffi::c_int) as isize) =
                        ' ' as ::core::ffi::c_int as schar_T;
                }
                *(*linebuf_char.ptr()).offset((rcol - 1 as ::core::ffi::c_int) as isize) =
                    if fcs_trunc != NUL as schar_T {
                        fcs_trunc
                    } else {
                        '>' as ::core::ffi::c_int as schar_T
                    };
                *(*linebuf_attr.ptr()).offset((rcol - 1 as ::core::ffi::c_int) as isize) =
                    trunc_attr as sattr_T;
            }
        }
        if pum_scrollbar.get() > 0 as ::core::ffi::c_int {
            let mut thumb: bool = i_0 >= thumb_pos && i_0 < thumb_pos + thumb_height;
            let mut scrollbar_col: ::core::ffi::c_int = col_off
                + (if pum_rl.get() as ::core::ffi::c_int != 0 {
                    -pum_width.get()
                } else {
                    pum_width.get()
                });
            let mut use_border_style: bool =
                has_border as ::core::ffi::c_int != 0 && !fconfig.shadow;
            grid_line_put_schar(
                scrollbar_col,
                if use_border_style as ::core::ffi::c_int != 0 && !thumb {
                    border_char
                } else {
                    fill_char
                },
                if thumb as ::core::ffi::c_int != 0 {
                    attr_thumb
                } else if use_border_style as ::core::ffi::c_int != 0 {
                    border_attr
                } else {
                    attr_scroll
                },
            );
        }
        grid_line_flush();
        row += 1;
        i_0 += 1;
    }
}
unsafe extern "C" fn pum_preview_set_text(
    mut win: *mut win_T,
    mut info: *mut ::core::ffi::c_char,
    mut lnum: *mut linenr_T,
    mut max_width: *mut ::core::ffi::c_int,
) {
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut replacement: Array = ARRAY_DICT_INIT;
    let mut buf: *mut buf_T = (*win).w_buffer;
    (*buf).b_p_ma = true_0;
    let mut curr: *mut ::core::ffi::c_char = info;
    let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    while !curr.is_null() {
        next = strchr(curr, '\n' as ::core::ffi::c_int);
        if !next.is_null() {
            *next = NUL as ::core::ffi::c_char;
        }
        if *curr as ::core::ffi::c_int == NUL && next.is_null() {
            break;
        }
        let mut save_wrap: bool = (*win).w_onebuf_opt.wo_wrap != 0;
        (*win).w_onebuf_opt.wo_wrap = false_0;
        let mut line_width: ::core::ffi::c_int =
            win_linetabsize(win, 0 as linenr_T, curr, MAXCOL as ::core::ffi::c_int);
        (*win).w_onebuf_opt.wo_wrap = save_wrap as ::core::ffi::c_int;
        *max_width = if *max_width > line_width {
            *max_width
        } else {
            line_width
        };
        if replacement.size == replacement.capacity {
            replacement.capacity = if replacement.capacity != 0 {
                replacement.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            replacement.items = xrealloc(
                replacement.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<Object>().wrapping_mul(replacement.capacity),
            ) as *mut Object;
        } else {
        };
        let c2rust_fresh5 = replacement.size;
        replacement.size = replacement.size.wrapping_add(1);
        *replacement.items.offset(c2rust_fresh5 as isize) = object {
            type_0: kObjectTypeString,
            data: C2Rust_Unnamed_12 {
                string: cstr_to_string(curr),
            },
        };
        *lnum += 1;
        if !next.is_null() {
            *next = '\n' as ::core::ffi::c_char;
        }
        curr = if !next.is_null() {
            next.offset(1 as ::core::ffi::c_int as isize)
        } else {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        };
    }
    let mut original_textlock: ::core::ffi::c_int = textlock.get();
    textlock.set(0 as ::core::ffi::c_int);
    nvim_buf_set_lines(
        0 as uint64_t,
        (*buf).handle as Buffer,
        0 as Integer,
        -1 as Integer,
        false_0 != 0,
        replacement,
        &raw mut arena,
        &raw mut err,
    );
    textlock.set(original_textlock);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        emsg(err.msg);
        api_clear_error(&raw mut err);
    }
    arena_mem_free(arena_finish(&raw mut arena));
    api_free_array(replacement);
    (*buf).b_p_ma = false_0;
}
unsafe extern "C" fn pum_adjust_info_position(
    mut wp: *mut win_T,
    mut width: ::core::ffi::c_int,
) -> bool {
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    let mut col: ::core::ffi::c_int = pum_col.get()
        + pum_width.get()
        + 1 as ::core::ffi::c_int
        + (if border_width > pum_scrollbar.get() {
            border_width
        } else {
            pum_scrollbar.get()
        });
    let mut right_extra: ::core::ffi::c_int = Columns.get() - col;
    let mut left_extra: ::core::ffi::c_int = pum_col.get() - 2 as ::core::ffi::c_int;
    let mut max_extra: ::core::ffi::c_int = if right_extra > left_extra {
        right_extra
    } else {
        left_extra
    };
    if max_extra < 10 as ::core::ffi::c_int {
        (*wp).w_config.hide = true_0 != 0;
        return false_0 != 0;
    }
    if right_extra > width {
        (*wp).w_config.width = width;
        (*wp).w_config.col = (col - 1 as ::core::ffi::c_int) as ::core::ffi::c_double;
    } else if left_extra > width {
        (*wp).w_config.width = width;
        (*wp).w_config.col = (pum_col.get() - (*wp).w_config.width - 1 as ::core::ffi::c_int)
            as ::core::ffi::c_double;
    } else {
        let place_in_right: bool = right_extra > left_extra;
        (*wp).w_config.width = max_extra;
        (*wp).w_config.col = (if place_in_right as ::core::ffi::c_int != 0 {
            col - 1 as ::core::ffi::c_int
        } else {
            pum_col.get() - (*wp).w_config.width - 1 as ::core::ffi::c_int
        }) as ::core::ffi::c_double;
    }
    (*wp).w_config.anchor = 0 as ::core::ffi::c_int as FloatAnchor;
    let mut count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
    (*wp).w_view_width = (*wp).w_config.width;
    (*wp).w_config.height = plines_m_win(wp, (*wp).w_topline, count, Rows.get());
    (*wp).w_config.row = pum_row.get() as ::core::ffi::c_double;
    (*wp).w_config.hide = false_0 != 0;
    win_config_float(wp, (*wp).w_config);
    return true_0 != 0;
}
pub unsafe extern "C" fn pum_set_info(
    mut selected: ::core::ffi::c_int,
    mut info: *mut ::core::ffi::c_char,
) -> *mut win_T {
    if !pum_is_visible.get() || !compl_match_curr_select(selected) {
        return ::core::ptr::null_mut::<win_T>();
    }
    block_autocmds();
    (*RedrawingDisabled.ptr()) += 1;
    (*no_u_sync.ptr()) += 1;
    let mut wp: *mut win_T = win_float_find_preview();
    if wp.is_null() {
        wp = win_float_create_preview(false_0 != 0, true_0 != 0);
        if wp.is_null() {
            return ::core::ptr::null_mut::<win_T>();
        }
        (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*wp).w_onebuf_opt.wo_wfb = true_0;
    }
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut max_info_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    pum_preview_set_text(wp, info, &raw mut lnum, &raw mut max_info_width);
    (*no_u_sync.ptr()) -= 1;
    (*RedrawingDisabled.ptr()) -= 1;
    redraw_later(wp, UPD_NOT_VALID);
    if !pum_adjust_info_position(wp, max_info_width) {
        wp = ::core::ptr::null_mut::<win_T>();
    }
    unblock_autocmds();
    return wp;
}
unsafe extern "C" fn pum_set_selected(
    mut n: ::core::ffi::c_int,
    mut repeat: ::core::ffi::c_int,
) -> bool {
    let mut resized: bool = false_0 != 0;
    let mut context: ::core::ffi::c_int = pum_height.get() / 2 as ::core::ffi::c_int;
    let mut prev_selected: ::core::ffi::c_int = pum_selected.get();
    pum_selected.set(n);
    let mut scroll_offset: ::core::ffi::c_int = pum_selected.get() - pum_height.get();
    let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
    let mut use_float: bool = cur_cot_flags
        & kOptCotFlagPopup as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0 as ::core::ffi::c_uint;
    if use_float as ::core::ffi::c_int != 0
        && (pum_selected.get() < 0 as ::core::ffi::c_int
            || (*(*pum_array.ptr()).offset(pum_selected.get() as isize))
                .pum_info
                .is_null())
    {
        let mut wp: *mut win_T = win_float_find_preview();
        if !wp.is_null() {
            (*wp).w_config.hide = true_0 != 0;
            win_config_float(wp, (*wp).w_config);
        }
    }
    if pum_selected.get() >= 0 as ::core::ffi::c_int && pum_selected.get() < pum_size.get() {
        if pum_first.get() > pum_selected.get() - 4 as ::core::ffi::c_int {
            if pum_first.get() > pum_selected.get() - 2 as ::core::ffi::c_int {
                (*pum_first.ptr()) -= pum_height.get() - 2 as ::core::ffi::c_int;
                if pum_first.get() < 0 as ::core::ffi::c_int {
                    pum_first.set(0 as ::core::ffi::c_int);
                } else if pum_first.get() > pum_selected.get() {
                    pum_first.set(pum_selected.get());
                }
            } else {
                pum_first.set(pum_selected.get());
            }
        } else if pum_first.get() < scroll_offset + 5 as ::core::ffi::c_int {
            if pum_first.get() < scroll_offset + 3 as ::core::ffi::c_int {
                pum_first.set(
                    if pum_first.get() + pum_height.get() - 2 as ::core::ffi::c_int
                        > scroll_offset + 1 as ::core::ffi::c_int
                    {
                        pum_first.get() + pum_height.get() - 2 as ::core::ffi::c_int
                    } else {
                        scroll_offset + 1 as ::core::ffi::c_int
                    },
                );
            } else {
                pum_first.set(scroll_offset + 1 as ::core::ffi::c_int);
            }
        }
        context = if context < 3 as ::core::ffi::c_int {
            context
        } else {
            3 as ::core::ffi::c_int
        };
        if pum_height.get() > 2 as ::core::ffi::c_int {
            if pum_first.get() > pum_selected.get() - context {
                pum_first.set(if pum_selected.get() - context > 0 as ::core::ffi::c_int {
                    pum_selected.get() - context
                } else {
                    0 as ::core::ffi::c_int
                });
            } else if pum_first.get()
                < pum_selected.get() + context - pum_height.get() + 1 as ::core::ffi::c_int
            {
                pum_first
                    .set(pum_selected.get() + context - pum_height.get() + 1 as ::core::ffi::c_int);
            }
        }
        pum_first.set(if pum_first.get() < pum_size.get() - pum_height.get() {
            pum_first.get()
        } else {
            pum_size.get() - pum_height.get()
        });
        if !(*(*pum_array.ptr()).offset(pum_selected.get() as isize))
            .pum_info
            .is_null()
            && Rows.get() > 10 as ::core::ffi::c_int
            && repeat <= 1 as ::core::ffi::c_int
            && cur_cot_flags
                & (kOptCotFlagPreview as ::core::ffi::c_int
                    | kOptCotFlagPopup as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                != 0
            && !(cur_cot_flags & kOptCotFlagPreview as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0
                && cmdwin_type.get() != 0 as ::core::ffi::c_int)
        {
            let mut curwin_save: *mut win_T = curwin.get();
            let mut curtab_save: *mut tabpage_T = curtab.get();
            if use_float {
                block_autocmds();
            }
            g_do_tagpreview.set(3 as ::core::ffi::c_int);
            if p_pvh.get() > 0 as OptInt && p_pvh.get() < g_do_tagpreview.get() as OptInt {
                g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
            }
            (*RedrawingDisabled.ptr()) += 1;
            (*no_u_sync.ptr()) += 1;
            if !use_float {
                resized = prepare_tagpreview(false_0 != 0);
            } else {
                let mut wp_0: *mut win_T = win_float_find_preview();
                if !wp_0.is_null() {
                    win_enter(wp_0, false_0 != 0);
                } else {
                    wp_0 = win_float_create_preview(true_0 != 0, true_0 != 0);
                    if !wp_0.is_null() {
                        resized = true_0 != 0;
                    }
                }
            }
            (*no_u_sync.ptr()) -= 1;
            (*RedrawingDisabled.ptr()) -= 1;
            g_do_tagpreview.set(0 as ::core::ffi::c_int);
            if (*curwin.get()).w_onebuf_opt.wo_pvw != 0
                || (*curwin.get()).w_float_is_info as ::core::ffi::c_int != 0
            {
                let mut res: ::core::ffi::c_int = OK;
                if !resized
                    && (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
                    && (*curbuf.get()).b_fname.is_null()
                    && bt_nofile(curbuf.get()) as ::core::ffi::c_int != 0
                    && *(*curbuf.get())
                        .b_p_bh
                        .offset(0 as ::core::ffi::c_int as isize)
                        as ::core::ffi::c_int
                        == 'w' as ::core::ffi::c_int
                {
                    buf_clear();
                } else {
                    (*no_u_sync.ptr()) += 1;
                    res = do_ecmd(
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        ::core::ptr::null_mut::<exarg_T>(),
                        ECMD_ONE as ::core::ffi::c_int as linenr_T,
                        0 as ::core::ffi::c_int,
                        ::core::ptr::null_mut::<win_T>(),
                    );
                    (*no_u_sync.ptr()) -= 1;
                    if res == OK {
                        set_option_value_give_err(
                            kOptSwapfile,
                            OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData { boolean: kFalse },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptBuflisted,
                            OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData { boolean: kFalse },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptBuftype,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: String_0 {
                                        data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                        size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>()
                                            .wrapping_sub(1 as size_t),
                                    },
                                },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptBufhidden,
                            OptVal {
                                type_0: kOptValTypeString,
                                data: OptValData {
                                    string: String_0 {
                                        data: b"wipe\0".as_ptr() as *const ::core::ffi::c_char
                                            as *mut ::core::ffi::c_char,
                                        size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>()
                                            .wrapping_sub(1 as size_t),
                                    },
                                },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                        set_option_value_give_err(
                            kOptDiff,
                            OptVal {
                                type_0: kOptValTypeBoolean,
                                data: OptValData { boolean: kFalse },
                            },
                            OPT_LOCAL as ::core::ffi::c_int,
                        );
                    }
                }
                if res == OK {
                    let mut lnum: linenr_T = 0 as linenr_T;
                    let mut max_info_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    pum_preview_set_text(
                        curwin.get(),
                        (*(*pum_array.ptr()).offset(pum_selected.get() as isize)).pum_info,
                        &raw mut lnum,
                        &raw mut max_info_width,
                    );
                    if repeat == 0 as ::core::ffi::c_int && !use_float {
                        lnum = if lnum < p_pvh.get() as linenr_T {
                            lnum
                        } else {
                            p_pvh.get() as linenr_T
                        };
                        if ((*curwin.get()).w_height as linenr_T) < lnum {
                            win_setheight(lnum as ::core::ffi::c_int);
                            resized = true_0 != 0;
                        }
                    }
                    (*curbuf.get()).b_changed = false_0;
                    (*curbuf.get()).b_p_ma = false_0;
                    if pum_selected.get() != prev_selected {
                        (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
                    } else if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
                        (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
                    }
                    (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                    if use_float {
                        if !pum_adjust_info_position(curwin.get(), max_info_width)
                            && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        {
                            win_enter(curwin_save, false_0 != 0);
                        }
                    }
                    if curwin.get() != curwin_save
                        && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        || curtab.get() != curtab_save
                            && valid_tabpage(curtab_save) as ::core::ffi::c_int != 0
                    {
                        if curtab.get() != curtab_save
                            && valid_tabpage(curtab_save) as ::core::ffi::c_int != 0
                        {
                            goto_tabpage_tp(curtab_save, false_0 != 0, false_0 != 0);
                        }
                        if ins_compl_active() as ::core::ffi::c_int != 0 && !resized {
                            (*curwin.get()).w_redr_status = false_0 != 0;
                        }
                        validate_cursor(curwin.get());
                        redraw_later(curwin.get(), UPD_SOME_VALID);
                        if resized as ::core::ffi::c_int != 0
                            && win_valid(curwin_save) as ::core::ffi::c_int != 0
                        {
                            (*no_u_sync.ptr()) += 1;
                            win_enter(curwin_save, true_0 != 0);
                            (*no_u_sync.ptr()) -= 1;
                            update_topline(curwin.get());
                        }
                        pum_is_visible.set(false_0 != 0);
                        update_screen();
                        pum_is_visible.set(true_0 != 0);
                        if !resized && win_valid(curwin_save) as ::core::ffi::c_int != 0 {
                            (*no_u_sync.ptr()) += 1;
                            win_enter(curwin_save, true_0 != 0);
                            (*no_u_sync.ptr()) -= 1;
                        }
                        pum_is_visible.set(false_0 != 0);
                        update_screen();
                        pum_is_visible.set(true_0 != 0);
                    }
                }
            }
            if use_float {
                unblock_autocmds();
            }
        }
    }
    return resized;
}
pub unsafe extern "C" fn pum_undisplay(mut immediate: bool) {
    pum_is_visible.set(false_0 != 0);
    pum_array.set(::core::ptr::null_mut::<pumitem_T>());
    must_redraw_pum.set(false_0 != 0);
    if immediate {
        pum_check_clear();
    }
}
pub unsafe extern "C" fn pum_check_clear() {
    if !pum_is_visible.get() && pum_is_drawn.get() as ::core::ffi::c_int != 0 {
        if pum_external.get() {
            ui_call_popupmenu_hide();
        } else {
            ui_comp_remove_grid(pum_grid.ptr());
            if ui_has(kUIMultigrid) {
                ui_call_win_close((*pum_grid.ptr()).handle as Integer);
                ui_call_grid_destroy((*pum_grid.ptr()).handle as Integer);
            }
            grid_free(pum_grid.ptr());
        }
        pum_is_drawn.set(false_0 != 0);
        pum_external.set(false_0 != 0);
        let mut wp: *mut win_T = win_float_find_preview();
        if !wp.is_null() {
            win_close(wp, false_0 != 0, false_0 != 0);
        }
    }
}
pub unsafe extern "C" fn pum_clear() {
    pum_first.set(0 as ::core::ffi::c_int);
}
pub unsafe extern "C" fn pum_visible() -> bool {
    return pum_is_visible.get();
}
pub unsafe extern "C" fn pum_drawn() -> bool {
    return pum_visible() as ::core::ffi::c_int != 0 && !pum_external.get();
}
pub unsafe extern "C" fn pum_invalidate() {
    pum_invalid.set(true_0 != 0);
}
pub unsafe extern "C" fn pum_ext_select_item(
    mut item: ::core::ffi::c_int,
    mut insert: bool,
    mut finish: bool,
) {
    if !pum_visible() || item < -1 as ::core::ffi::c_int || item >= pum_size.get() {
        return;
    }
    (*pum_want.ptr()).active = true_0 != 0;
    (*pum_want.ptr()).item = item;
    (*pum_want.ptr()).insert = insert;
    (*pum_want.ptr()).finish = finish;
}
pub unsafe extern "C" fn pum_get_height() -> ::core::ffi::c_int {
    if pum_external.get() {
        let mut ui_pum_height: ::core::ffi::c_int = ui_pum_get_height();
        if ui_pum_height != 0 {
            return ui_pum_height;
        }
    }
    return pum_height.get();
}
pub unsafe extern "C" fn pum_set_event_info(mut dict: *mut dict_T) {
    if !pum_visible() {
        return;
    }
    let mut w: ::core::ffi::c_double = 0.;
    let mut h: ::core::ffi::c_double = 0.;
    let mut r: ::core::ffi::c_double = 0.;
    let mut c: ::core::ffi::c_double = 0.;
    if !ui_pum_get_pos(&raw mut w, &raw mut h, &raw mut r, &raw mut c) {
        w = pum_width.get() as ::core::ffi::c_double;
        h = pum_height.get() as ::core::ffi::c_double;
        r = pum_row.get() as ::core::ffi::c_double;
        c = pum_col.get() as ::core::ffi::c_double;
    }
    tv_dict_add_float(
        dict,
        b"height\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
        h as float_T,
    );
    tv_dict_add_float(
        dict,
        b"width\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        w as float_T,
    );
    tv_dict_add_float(
        dict,
        b"row\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        r as float_T,
    );
    tv_dict_add_float(
        dict,
        b"col\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 4]>().wrapping_sub(1 as size_t),
        c as float_T,
    );
    tv_dict_add_nr(
        dict,
        b"size\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        pum_size.get() as varnumber_T,
    );
    tv_dict_add_bool(
        dict,
        b"scrollbar\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
        (if pum_scrollbar.get() != 0 {
            kBoolVarTrue as ::core::ffi::c_int
        } else {
            kBoolVarFalse as ::core::ffi::c_int
        }) as BoolVarValue,
    );
}
unsafe extern "C" fn pum_position_at_mouse(mut min_width: ::core::ffi::c_int) {
    let mut min_row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut min_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut max_row: ::core::ffi::c_int = Rows.get();
    let mut max_col: ::core::ffi::c_int = Columns.get();
    let mut grid: ::core::ffi::c_int = mouse_grid.get();
    let mut row: ::core::ffi::c_int = mouse_row.get();
    let mut col: ::core::ffi::c_int = mouse_col.get();
    pum_win_row_offset.set(0 as ::core::ffi::c_int);
    pum_win_col_offset.set(0 as ::core::ffi::c_int);
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 && grid == 0 as ::core::ffi::c_int {
        mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
    }
    if grid > 1 as ::core::ffi::c_int {
        let mut wp: *mut win_T = get_win_by_grid_handle(grid as handle_T);
        if !wp.is_null() {
            row += (*wp).w_winrow;
            col += (*wp).w_wincol;
            pum_win_row_offset.set((*wp).w_winrow);
            pum_win_col_offset.set((*wp).w_wincol);
            if (*wp).w_view_height > 0 as ::core::ffi::c_int
                || (*wp).w_view_width > 0 as ::core::ffi::c_int
            {
                max_row = if Rows.get() - (*wp).w_winrow > (*wp).w_winrow + (*wp).w_view_height {
                    Rows.get() - (*wp).w_winrow
                } else {
                    (*wp).w_winrow + (*wp).w_view_height
                };
                max_col = if Columns.get() - (*wp).w_wincol > (*wp).w_wincol + (*wp).w_view_width {
                    Columns.get() - (*wp).w_wincol
                } else {
                    (*wp).w_wincol + (*wp).w_view_width
                };
            }
        }
    }
    if (*pum_grid.ptr()).handle != 0 as ::core::ffi::c_int && grid == (*pum_grid.ptr()).handle {
        row += pum_row.get();
        col += pum_left_col.get();
    } else {
        pum_anchor_grid.set(grid);
    }
    let mut border_width: ::core::ffi::c_int = pum_border_width();
    let mut border_height: ::core::ffi::c_int = border_width;
    if max_row - row > pum_size.get() + border_height || max_row - row > row - min_row {
        pum_above.set(false_0 != 0);
        pum_row.set(row + 1 as ::core::ffi::c_int);
        if pum_height.get() + border_height > max_row - pum_row.get() {
            pum_height.set(max_row - pum_row.get() - border_height);
        }
    } else {
        pum_above.set(true_0 != 0);
        pum_row.set(row - pum_size.get() - border_height);
        if pum_row.get() < min_row {
            (*pum_height.ptr()) += pum_row.get() - min_row;
            pum_row.set(min_row);
        }
    }
    if pum_rl.get() {
        if col - min_col + 1 as ::core::ffi::c_int >= pum_base_width.get() + border_width
            || col - min_col + 1 as ::core::ffi::c_int > min_width + border_width
        {
            pum_col.set(col);
        } else {
            pum_col.set(
                min_col
                    + (if pum_base_width.get() + border_width < min_width + border_width {
                        pum_base_width.get() + border_width
                    } else {
                        min_width + border_width
                    })
                    - 1 as ::core::ffi::c_int,
            );
        }
        pum_width.set(pum_col.get() - min_col + 1 as ::core::ffi::c_int - border_width);
    } else {
        if max_col - col >= pum_base_width.get() + border_width
            || max_col - col > min_width + border_width
        {
            pum_col.set(col);
        } else {
            pum_col.set(
                max_col
                    - (if pum_base_width.get() + border_width < min_width + border_width {
                        pum_base_width.get() + border_width
                    } else {
                        min_width + border_width
                    }),
            );
        }
        pum_width.set(max_col - pum_col.get() - border_width);
    }
    pum_width.set(
        if pum_width.get() < pum_base_width.get() + 1 as ::core::ffi::c_int {
            pum_width.get()
        } else {
            pum_base_width.get() + 1 as ::core::ffi::c_int
        },
    );
}
unsafe extern "C" fn pum_select_mouse_pos() {
    let mut grid: ::core::ffi::c_int = mouse_grid.get();
    let mut row: ::core::ffi::c_int = mouse_row.get();
    let mut col: ::core::ffi::c_int = mouse_col.get();
    if grid == 0 as ::core::ffi::c_int {
        mouse_find_win_outer(&raw mut grid, &raw mut row, &raw mut col);
    }
    if grid == (*pum_grid.ptr()).handle {
        let mut border_offset: ::core::ffi::c_int = if pum_border_width() == 2 as ::core::ffi::c_int
        {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
        let mut item: ::core::ffi::c_int = row - border_offset;
        pum_selected.set(
            if item >= 0 as ::core::ffi::c_int && item < pum_height.get() {
                item
            } else {
                -1 as ::core::ffi::c_int
            },
        );
        return;
    }
    if grid != pum_anchor_grid.get()
        || col < pum_left_col.get() - pum_win_col_offset.get()
        || col >= pum_right_col.get() - pum_win_col_offset.get()
    {
        pum_selected.set(-1 as ::core::ffi::c_int);
        return;
    }
    let mut idx: ::core::ffi::c_int = row - (pum_row.get() - pum_win_row_offset.get());
    if idx < 0 as ::core::ffi::c_int || idx >= pum_height.get() {
        pum_selected.set(-1 as ::core::ffi::c_int);
    } else if *(*(*pum_array.ptr()).offset(idx as isize)).pum_text as ::core::ffi::c_int != NUL {
        pum_selected.set(idx);
    }
}
unsafe extern "C" fn pum_execute_menu(mut menu: *mut vimmenu_T, mut mode: ::core::ffi::c_int) {
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut ea: exarg_T = exarg_T {
        arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_append,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: 0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    let mut mp: *mut vimmenu_T = (*menu).children;
    while !mp.is_null() {
        if (*mp).modes & (*mp).enabled & mode != 0 && {
            let c2rust_fresh7 = idx;
            idx = idx + 1;
            c2rust_fresh7 == pum_selected.get()
        } {
            memset(
                &raw mut ea as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<exarg_T>(),
            );
            execute_menu(&raw mut ea, mp, -1 as ::core::ffi::c_int);
            break;
        } else {
            mp = (*mp).next;
        }
    }
}
pub unsafe extern "C" fn pum_show_popupmenu(mut menu: *mut vimmenu_T) {
    pum_undisplay(true_0 != 0);
    pum_size.set(0 as ::core::ffi::c_int);
    let mut mode: ::core::ffi::c_int = get_menu_mode_flag();
    let mut mp: *mut vimmenu_T = (*menu).children;
    while !mp.is_null() {
        if menu_is_separator((*mp).dname) as ::core::ffi::c_int != 0
            || (*mp).modes & (*mp).enabled & mode != 0
        {
            (*pum_size.ptr()) += 1;
        }
        mp = (*mp).next;
    }
    if pum_size.get() <= 0 as ::core::ffi::c_int {
        emsg(gettext(
            &raw const e_menu_only_exists_in_another_mode as *const ::core::ffi::c_char,
        ));
        return;
    }
    let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut array: *mut pumitem_T = xcalloc(
        pum_size.get() as size_t,
        ::core::mem::size_of::<pumitem_T>(),
    ) as *mut pumitem_T;
    let mut mp_0: *mut vimmenu_T = (*menu).children;
    while !mp_0.is_null() {
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if menu_is_separator((*mp_0).dname) {
            s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else if (*mp_0).modes & (*mp_0).enabled & mode != 0 {
            s = (*mp_0).dname;
        }
        if !s.is_null() {
            s = xstrdup(s);
            let c2rust_fresh6 = idx;
            idx = idx + 1;
            let c2rust_lvalue_ptr = &raw mut (*array.offset(c2rust_fresh6 as isize)).pum_text;
            *c2rust_lvalue_ptr = s;
        }
        mp_0 = (*mp_0).next;
    }
    pum_array.set(array);
    pum_compute_size();
    pum_scrollbar.set(0 as ::core::ffi::c_int);
    pum_height.set(pum_size.get());
    pum_rl.set((*curwin.get()).w_onebuf_opt.wo_rl != 0);
    pum_position_at_mouse(20 as ::core::ffi::c_int);
    pum_selected.set(-1 as ::core::ffi::c_int);
    pum_first.set(0 as ::core::ffi::c_int);
    if p_mousemev.get() == 0 {
        ui_call_option_set(
            String_0 {
                data: b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
            },
            object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 { boolean: true },
            },
        );
    }
    loop {
        pum_is_visible.set(true_0 != 0);
        pum_is_drawn.set(true_0 != 0);
        (*pum_grid.ptr()).zindex = kZIndexCmdlinePopupMenu as ::core::ffi::c_int;
        pum_redraw();
        setcursor_mayforce(curwin.get(), true_0 != 0);
        let mut c: ::core::ffi::c_int = vgetc();
        if c == ESC || c == Ctrl_C || (*pum_array.ptr()).is_null() {
            break;
        }
        if c == CAR || c == NL {
            pum_execute_menu(menu, mode);
            break;
        } else if c == 'k' as ::core::ffi::c_int
            || c == K_UP
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            while pum_selected.get() > 0 as ::core::ffi::c_int {
                (*pum_selected.ptr()) -= 1;
                if *(*array.offset(pum_selected.get() as isize)).pum_text as ::core::ffi::c_int
                    != NUL
                {
                    break;
                }
            }
        } else if c == 'j' as ::core::ffi::c_int
            || c == K_DOWN
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            while pum_selected.get() < pum_size.get() - 1 as ::core::ffi::c_int {
                (*pum_selected.ptr()) += 1;
                if *(*array.offset(pum_selected.get() as isize)).pum_text as ::core::ffi::c_int
                    != NUL
                {
                    break;
                }
            }
        } else if c
            == -(253 as ::core::ffi::c_int
                + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            vungetc(c);
            break;
        } else if c
            == -(253 as ::core::ffi::c_int
                + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            || c == -(253 as ::core::ffi::c_int
                + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            pum_select_mouse_pos();
        } else {
            if !(c
                == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
            {
                continue;
            }
            pum_select_mouse_pos();
            if pum_selected.get() >= 0 as ::core::ffi::c_int {
                pum_execute_menu(menu, mode);
                break;
            } else if c
                == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                break;
            }
        }
    }
    idx = 0 as ::core::ffi::c_int;
    while idx < pum_size.get() {
        xfree((*array.offset(idx as isize)).pum_text as *mut ::core::ffi::c_void);
        idx += 1;
    }
    xfree(array as *mut ::core::ffi::c_void);
    pum_undisplay(true_0 != 0);
    if p_mousemev.get() == 0 {
        ui_call_option_set(
            String_0 {
                data: b"mousemoveevent\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
                size: ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
            },
            object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_12 { boolean: false },
            },
        );
    }
}
pub unsafe extern "C" fn pum_make_popup(
    mut path_name: *const ::core::ffi::c_char,
    mut use_mouse_pos: ::core::ffi::c_int,
) {
    if use_mouse_pos == 0 {
        mouse_row.set((*curwin.get()).w_grid.row_offset + (*curwin.get()).w_wrow);
        mouse_col.set(
            (*curwin.get()).w_grid.col_offset
                + (if (*curwin.get()).w_onebuf_opt.wo_rl != 0 {
                    (*curwin.get()).w_view_width - (*curwin.get()).w_wcol - 1 as ::core::ffi::c_int
                } else {
                    (*curwin.get()).w_wcol
                }),
        );
        if ui_has(kUIMultigrid) {
            mouse_grid.set((*(*curwin.get()).w_grid.target).handle as ::core::ffi::c_int);
        } else if (*curwin.get()).w_grid.target != default_grid.ptr() {
            mouse_grid.set(0 as ::core::ffi::c_int);
            (*mouse_row.ptr()) += (*curwin.get()).w_winrow;
            (*mouse_col.ptr()) += (*curwin.get()).w_wincol;
        }
    }
    let mut menu: *mut vimmenu_T = menu_find(path_name);
    if !menu.is_null() {
        pum_show_popupmenu(menu);
    }
}
pub unsafe extern "C" fn pum_ui_flush() {
    if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        && pum_is_drawn.get() as ::core::ffi::c_int != 0
        && !pum_external.get()
        && (*pum_grid.ptr()).handle != 0 as ::core::ffi::c_int
        && (*pum_grid.ptr()).pending_comp_index_update as ::core::ffi::c_int != 0
    {
        let mut anchor: *const ::core::ffi::c_char = if pum_above.get() as ::core::ffi::c_int != 0 {
            b"SW\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NW\0".as_ptr() as *const ::core::ffi::c_char
        };
        let mut row_off: ::core::ffi::c_int = if pum_above.get() as ::core::ffi::c_int != 0 {
            -pum_height.get()
        } else {
            0 as ::core::ffi::c_int
        };
        ui_call_win_float_pos(
            (*pum_grid.ptr()).handle as Integer,
            -1 as Window,
            cstr_as_string(anchor),
            pum_anchor_grid.get() as Integer,
            (pum_row.get() - row_off - pum_win_row_offset.get()) as Float,
            (pum_left_col.get() - pum_win_col_offset.get()) as Float,
            false_0 != 0,
            (*pum_grid.ptr()).zindex as Integer,
            (*pum_grid.ptr()).comp_index as ::core::ffi::c_int as Integer,
            (*pum_grid.ptr()).comp_row as Integer,
            (*pum_grid.ptr()).comp_col as Integer,
        );
        (*pum_grid.ptr()).pending_comp_index_update = false_0 != 0;
    }
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
