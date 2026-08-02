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

// The carve of the transpiled module; see each child's docs.
mod layout;
pub(crate) use self::layout::*;
mod draw;
pub use self::draw::*;
mod preview;
pub use self::preview::*;
mod menu;
pub use self::menu::*;
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
