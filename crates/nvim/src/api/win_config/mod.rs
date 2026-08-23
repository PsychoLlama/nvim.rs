#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::extmark::{parse_virt_text, virt_text_to_array};
use crate::api::private::helpers::{
    NIL, api_clear_error, api_free_array, api_free_object, api_set_error, api_typename,
    arena_array, cstr_as_string, cstr_to_string, cstrn_as_string, find_buffer_by_handle,
    find_window_by_handle, has_key, object_to_hl_id, try_enter, try_leave,
};
use crate::api::private::validate::{
    api_err_conflict, api_err_exp, api_err_invalid, api_err_required,
};
use crate::autocmd::{
    EVENT_WINNEW, apply_autocmds, block_autocmds, is_aucmd_win, unblock_autocmds,
};
use crate::buffer::{bufref_valid, set_bufref};
use crate::drawscreen::{UPD_NOT_VALID, redraw_later, set_must_redraw};
use crate::eval::window::{restore_win, restore_win_noblock, switch_win, switch_win_noblock};
use crate::ex_docmd::expr_map_locked;
use crate::global_cell::GlobalCell;
use crate::highlight_group::{syn_check_group, syn_id2name};
use crate::main::{
    autocmd_no_enter, autocmd_no_leave, cmdline_win, cmdwin_buf, cmdwin_old_curwin, cmdwin_type,
    cmdwin_win, curbuf, curtab, curwin, e_cmdwin, e_textlock, float_anchor_str, p_sb, p_spr,
    p_winborder, textlock,
};
use crate::mbyte::{mb_string2cells, mb_string2cells_len};
use crate::memory::{strequal, xstrdup};
use crate::r#move::changed_window_setting;
use crate::option::{copy_option_part, didset_window_options};
use crate::os::cshim::strchr;
use crate::strings::striequal;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    AlignTextPos, Arena, Array, Boolean, BorderTextType, Buffer, CMD_SIZE, Error, Float,
    FloatAnchor, FloatRelative, Integer, KeyDict_win_config, Object, OptionalKeys, String_0,
    TryState, VirtText, VirtTextChunk, WinConfig, WinSplit, WinStyle, Window, buf_T, bufref_T,
    colnr_T, except_T, frame_T, int64_t, kErrorTypeException, kErrorTypeNone, kErrorTypeValidation,
    kFloatAnchorEast, kFloatAnchorSouth, kFloatRelativeCursor, kFloatRelativeEditor,
    kFloatRelativeLaststatus, kFloatRelativeMouse, kFloatRelativeTabline, kFloatRelativeWindow,
    kObjectTypeArray, kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, linenr_T, lpos_T,
    msglist_T, object, object_data as C2Rust_Unnamed, size_t, switchwin_T, tabpage_T, win_T,
};
use crate::ui::ui_has;
use crate::ui_compositor::ui_comp_remove_grid;
use crate::window::{
    WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_NOENTER, WSP_TOP, WSP_VERT,
    check_split_disallowed_err, clear_float_config, goto_tabpage_win, last_status,
    lastwin_nofloating, merge_win_config, one_window, win_append, win_comp_pos, win_find_tabpage,
    win_goto, win_locked, win_remove, win_set_buf, win_setheight_win, win_setwidth_win,
    win_split_ins, win_valid, win_valid_any_tab, window_layout_locked_err, winframe_find_altwin,
    winframe_remove, winframe_restore,
};
use crate::winfloat::{
    win_config_float, win_float_find_altwin, win_new_float, win_set_minimal_style,
};

// The carve of the transpiled module; see each child's docs.
mod apply;
mod border;
mod get;
mod open;
mod parse;

pub use self::apply::*;
pub use self::border::*;
pub use self::get::*;
pub use self::open::*;
pub(crate) use self::parse::*;
pub const kAlignRight: AlignTextPos = 2;
pub const kAlignCenter: AlignTextPos = 1;
pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitBelow: WinSplit = 3;
pub const kWinSplitAbove: WinSplit = 2;
pub const kWinSplitRight: WinSplit = 1;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const kZIndexFloatDefault: C2Rust_Unnamed_13 = 50;
pub const kBorderTextFooter: BorderTextType = 1;
pub const kBorderTextTitle: BorderTextType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const KEYSET_OPTIDX_win_config__col: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__row: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__win: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__hide: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__width: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__split: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__title: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__mouse: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__fixed: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__style: ::core::ffi::c_int = 10 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__anchor: ::core::ffi::c_int = 11 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__bufpos: ::core::ffi::c_int = 12 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__height: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__zindex: ::core::ffi::c_int = 14 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__footer: ::core::ffi::c_int = 15 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__border: ::core::ffi::c_int = 16 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__external: ::core::ffi::c_int = 17 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__relative: ::core::ffi::c_int = 18 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__vertical: ::core::ffi::c_int = 19 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__focusable: ::core::ffi::c_int = 20 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__noautocmd: ::core::ffi::c_int = 21 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__title_pos: ::core::ffi::c_int = 22 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config__footer_pos: ::core::ffi::c_int = 23 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_win_config___cmdline_offset: ::core::ffi::c_int = 24 as ::core::ffi::c_int;
pub const KEYDICT_INIT: KeyDict_win_config = KeyDict_win_config {
    is_set__win_config_: 0 as OptionalKeys,
    external: false,
    fixed: false,
    focusable: false,
    footer: NIL,
    footer_pos: String_0::NULL,
    hide: false,
    height: 0,
    mouse: false,
    relative: String_0::NULL,
    row: 0.,
    style: String_0::NULL,
    noautocmd: false,
    vertical: false,
    win: 0,
    width: 0,
    zindex: 0,
    anchor: String_0::NULL,
    border: NIL,
    bufpos: Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    },
    col: 0.,
    split: String_0::NULL,
    title: NIL,
    title_pos: String_0::NULL,
    _cmdline_offset: 0,
};
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
