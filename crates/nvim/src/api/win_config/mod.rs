#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::extmark::{parse_virt_text, virt_text_to_array};
use crate::api::private::helpers::{
    api_typename, cstr_to_string, cstrn_to_string, find_buffer_by_handle, find_window_by_handle,
    object_to_hl_id, try_enter, try_leave,
};
use crate::autocmd::{apply_autocmds, block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::drawscreen::{UPD_NOT_VALID, redraw_later, set_must_redraw};
use crate::eval::window::{restore_win, restore_win_noblock, switch_win, switch_win_noblock};
use crate::ex_docmd::expr_map_locked;
use crate::guard::textlock;
use crate::highlight_group::{syn_check_group, syn_id2name};
use crate::mbyte::{mb_string2cells, mb_string2cells_len};
use crate::memory::xstrdup;
use crate::message::{e_cmdwin, e_textlock};
use crate::r#move::changed_window_setting;
use crate::option::vars::{p_sb, p_spr, p_winborder};
use crate::option::{copy_option_part, didset_window_options};
use crate::strings::striequal;
use crate::types::AutoEvent;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    AlignTextPos, Array, Boolean, BorderTextType, BufferHandle, ColNr, Error, FloatAnchor,
    FloatRelative, Integer, KeyDict_win_config, LPos, LineNr, Object, String_0, SwitchWin, Tabpage,
    TryState, VirtText, VirtTextChunk, WinConfig, WinSplit, WinStyle, WindowHandle,
    kErrorTypeException, kErrorTypeValidation, kFloatAnchorEast, kFloatAnchorSouth,
    kFloatRelativeCursor, kFloatRelativeEditor, kFloatRelativeLaststatus, kFloatRelativeMouse,
    kFloatRelativeTabline, kFloatRelativeWindow, size_t,
};
use crate::ui::ui_has;
use crate::ui_compositor::ui_comp_remove_grid;
use crate::window::{
    WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_NOENTER, WSP_TOP, WSP_VERT,
    check_split_disallowed_err, clear_float_config, find_altwin, goto_tabpage_win, last_status,
    lastwin_nofloating, merge_win_config, one_window, win_append, win_comp_pos, win_find_tabpage,
    win_goto, win_locked, win_remove, win_set_buf, win_setheight_win, win_setwidth_win,
    win_split_ins, win_valid, win_valid_any_tab, window_layout_locked_err, winframe_remove,
    winframe_restore,
};
use crate::winfloat::{
    win_config_float, win_float_find_altwin, win_new_float, win_set_minimal_style,
};
use crate::winlayer::graph::{cmdline_win, cmdwin_buf, cmdwin_old_curwin, cmdwin_type, cmdwin_win};

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
pub const kZIndexFloatDefault: ::core::ffi::c_uint = 50;
pub const kBorderTextFooter: BorderTextType = 1;
pub const kBorderTextTitle: BorderTextType = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: Array = Array::EMPTY;
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = KV_INITIAL_VALUE;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
