#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::api::extmark::{parse_virt_text, virt_text_to_array};
use crate::api::keysets::KeyDict_win_config;
use crate::api::private::helpers::TryState;
use crate::api::private::helpers::{
    api_typename, find_buffer_by_handle, find_window_by_handle, object_to_hl_id, try_enter,
    try_leave,
};
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::{block_autocmds, is_aucmd_win, unblock_autocmds};
use crate::decoration::defs::{VirtText, VirtTextChunk};
use crate::drawscreen::{UPD_NOT_VALID, redraw_later, set_must_redraw};
use crate::eval::window::{restore_win, restore_win_noblock, switch_win, switch_win_noblock};
use crate::ex_docmd::expr_map_locked;
use crate::grid::schar::MAX_SCHAR_SIZE;
use crate::guard::TEXTLOCK;
use crate::highlight_group::{syn_check_group, syn_id2name};
use crate::mbyte::{c_string_cells, c_string_cells_len};
use crate::message::{E_CMDWIN, E_TEXTLOCK};
use crate::r#move::changed_window_setting;
use crate::option::didset_window_options;
use crate::option::vars::{p_sb, p_spr, p_winborder};
use crate::types::buffer::{ALIGN_CENTER, ALIGN_RIGHT};
use crate::types::{
    AlignTextPos, ApiString, Array, Boolean, BorderTextType, BufferHandle, ERROR_TYPE_EXCEPTION,
    ERROR_TYPE_VALIDATION, Error, FLOAT_ANCHOR_EAST, FLOAT_ANCHOR_SOUTH, FLOAT_RELATIVE_CURSOR,
    FLOAT_RELATIVE_EDITOR, FLOAT_RELATIVE_LASTSTATUS, FLOAT_RELATIVE_MOUSE, FLOAT_RELATIVE_TABLINE,
    FLOAT_RELATIVE_WINDOW, FloatAnchor, FloatRelative, Integer, LPos, Object, SwitchWin, WinConfig,
    WinSplit, WinStyle, WindowHandle,
};
use crate::ui::state::UI_MULTIGRID;
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
use crate::winlayer::graph::{CMDLINE_WIN, CMDWIN_BUF, CMDWIN_OLD_CURWIN, CMDWIN_TYPE, CMDWIN_WIN};

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
pub const WIN_SPLIT_BELOW: WinSplit = 3;
pub const WIN_SPLIT_ABOVE: WinSplit = 2;
pub const WIN_SPLIT_RIGHT: WinSplit = 1;
pub const BORDER_TEXT_FOOTER: BorderTextType = 1;
pub const BORDER_TEXT_TITLE: BorderTextType = 0;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
