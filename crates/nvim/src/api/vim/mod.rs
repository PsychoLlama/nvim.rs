#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::api::buffer::{api_buf_ensure_loaded, nvim_buf_del_keymap};
use crate::api::deprecated::{buffer_del_line, buffer_get_line, buffer_set_line};
use crate::api::keysets::{
    KeyDict_complete_set, KeyDict_context, KeyDict_echo_opts, KeyDict_empty,
    KeyDict_eval_statusline, KeyDict_get_highlight, KeyDict_get_ns, KeyDict_highlight,
    KeyDict_keymap, KeyDict_open_term, KeyDict_redraw, KeyDict_runtime,
};
use crate::api::private::helpers::{
    api_metadata, api_set_sctx, api_typename, dict_get_value, dict_set_var, find_buffer_by_handle,
    find_window_by_handle, get_default_stl_hl, parse_hl_msg, set_mark, string_to_array,
};
use crate::autocmd::defs::AutoEvent;
use crate::autocmd::{
    apply_autocmds, block_autocmds, may_trigger_vim_suspend_resume, unblock_autocmds,
};
use crate::buffer::{
    buf_close_terminal, buf_get_changedtick, buflist_new, buflist_nr2name, do_buffer,
};
use crate::channel::{
    ChannelRef, alloc_channel, channel_all_info, channel_decref, channel_info, channel_send,
    with_channel,
};
use crate::context::Context;
use crate::context::{CTX_ALL, ctx_free, ctx_from_dict, ctx_restore, ctx_save, ctx_to_dict};
use crate::cursor::get_cursor_rel_lnum;
use crate::decoration::decor_redraw_signs;
use crate::drawline::use_cursor_line_highlight;
use crate::drawscreen::state::{MUST_REDRAW, REDRAW_TABLINE};
use crate::drawscreen::{
    UPD_CLEAR, UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_buf_later,
    redraw_buf_range_later, redraw_later, setcursor_mayforce, update_screen, win_update_cursorline,
};
use crate::eval::vars::{globvar_dict, set_vim_var_nr, vimvar_dict};
use crate::ex_docmd::state::EX_NORMAL_BUSY;
use crate::ex_docmd::{change_dir, exec_normal};
use crate::ex_eval::aborting;
use crate::ex_getln::state::CMDPREVIEW;
use crate::fold::FoldInfo;
use crate::fold::fold_info;
use crate::getchar::state::{TYPEBUF_WAS_FILLED, VGETC_BUSY};
use crate::getchar::{ins_typebuf, paste_store};
use crate::global_cell::GlobalCell;
use crate::grid::{
    GridRef, SCHAR_BUF_SIZE, SCharBuf, get_win_by_grid_handle, schar_cache_clear, schar_text,
    win_grid_alloc,
};
use crate::highlight::HlAttrs;
use crate::highlight::state::{NS_HL_FAST, NS_HL_GLOBAL};
use crate::highlight::{
    dict2hlattrs, highlight_use_hlstate, hl_check_ns, hl_get_attr_by_id, hl_inspect,
    hl_ns_get_attrs, ns_hl_def, win_check_ns_hl,
};
use crate::highlight_group::{
    COLOR_NAMES, HLF_CLN, HLF_CLS, HLF_LNA, HLF_LNB, HLF_N, HLF_SC, name_to_color, ns_get_hl_defs,
    syn_check_group, syn_id2name,
};
use crate::insexpand::get_cot_flags;
use crate::keycodes::{name_to_mod_mask, replace_termcodes};
use crate::log::G_STATS;
use crate::log::LOGLVL_DBG;
use crate::lua::executor::LuaRetMode;
use crate::lua::executor::{nlua_exec, nlua_get_global_ref_count, nlua_is_deferred_safe};
use crate::lua::state::TSLUA_QUERY_PARSE_COUNT;
use crate::mapping::{keymap_array, modify_keymap};
use crate::mark::mark_get_global;
use crate::mbyte::{c_cluster_len, c_cluster_schar, c_string_cells};
use crate::memline::ml_open;
use crate::memory::ARENA_ALLOC_COUNT;
use crate::message::MessageData;
use crate::message::state::{
    DID_EMSG, LINES_LEFT, MSG_DIDANY, MSG_NO_MORE, MSG_SCROLL, NEED_WAIT_RETURN,
};
use crate::message::{E_CMDWIN, E_INVCHAN};
use crate::message::{
    do_autocmd_progress, msg_id_exists, msg_multihl, verbose_enter, verbose_leave, verbose_stop,
};
use crate::r#move::{changed_window_setting, update_topline, validate_cursor};
use crate::mpack::typedefs::mpack_token_type_t;
use crate::msgpack_rpc::channel::rpc_set_client_info;
use crate::msgpack_rpc::unpacker::unpack;
use crate::normal::defs::MotionType;
use crate::normal::reset_visual_and_resel;
use crate::option::defs::OptVal;
use crate::option::vars::{P_LZ, p_cpo, p_lz};
use crate::option::{OptionTarget, buf_copy_options, set_option_direct_for};
use crate::options::{OPT_BUFHIDDEN, OPT_BUFTYPE, OPT_COT_FLAG_POPUP, OPT_INVALID};
use crate::optionstr::check_stl_option;
use crate::os::input::{input_blocking, input_enqueue, input_enqueue_mouse, input_enqueue_raw};
use crate::os::libc::{int64_t, size_t, uint16_t, uint64_t};
use crate::os::proc::os_proc_children;
use crate::popupmenu::{pum_ext_select_item, pum_set_info};
use crate::register::defs::YankReg;
use crate::register::{do_put, finish_yankreg_from_object, prepare_yankreg_from_object};
use crate::runtime::{
    RuntimeOpts, do_in_runtimepath_visit, do_source, get_lib_dir, runtime_get_named,
    runtime_inspect, script_autoload_named,
};
use crate::search::{BACKWARD, FORWARD};
use crate::sign::SignTextAttrs;
use crate::state::get_mode;
use crate::statusline::defs::StatusCol;
use crate::statusline::{draw_tabline, win_redr_status, win_redr_winbar};
use crate::terminal::state::TerminalOptions;
use crate::terminal::{
    buf_terminal_running, buf_terminal_set_streamed_paste, terminal_alloc, terminal_buf,
    terminal_check_size, terminal_destroy, terminal_open,
};
use crate::types::{
    ApiDict, ApiString, Array, Boolean, BufferHandle, CD_SCOPE_GLOBAL, ERROR_TYPE_EXCEPTION,
    ERROR_TYPE_VALIDATION, Error, Float, Handle, Integer, LineNr, LuaRef, MessageType,
    OBJECT_TYPE_STRING, Object, ScreenChar, ScriptId, TabpageHandle, VarNumber, Vv, WindowHandle,
};
use crate::ui::state::COLUMNS;
use crate::ui::{ui_array, ui_call_screenshot, ui_flush};
use crate::window::{goto_tabpage_tp, goto_tabpage_win, win_find_tabpage};
use crate::winlayer::graph::CMDWIN_BUF;

// The carve of the transpiled module; see each child's docs.
mod client;
mod context;
mod echo;
mod handles;
mod highlight;
mod input;
mod inspect;
mod marks;
mod paste;
mod redraw;
mod runtime;
mod statusline;
mod term;
mod vars;

pub use self::client::*;
pub use self::context::*;
pub use self::echo::*;
pub use self::handles::*;
pub use self::highlight::*;
pub use self::input::*;
pub use self::inspect::*;
pub use self::marks::*;
pub use self::paste::*;
pub use self::redraw::*;
pub use self::runtime::*;
pub use self::statusline::*;
pub use self::term::*;
pub use self::vars::*;
pub const MESSAGE_TYPE_NOTIFICATION: MessageType = 2;
pub const REPTERM_NO_SPECIAL: ::core::ffi::c_uint = 4;
pub const REPTERM_DO_LT: ::core::ffi::c_uint = 2;
pub const REPTERM_FROM_PART: ::core::ffi::c_uint = 1;
pub const RET_NIL_BOOL: LuaRetMode = 1;
pub const RET_OBJECT: LuaRetMode = 0;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const MT_CHAR_WISE: MotionType = 0;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
/// The top bit of a channel id: an API call made by the editor itself
/// rather than by a client.
pub const INTERNAL_CALL_MASK: uint64_t = 1 << (::core::mem::size_of::<uint64_t>() * 8 - 1);
#[inline(always)]
fn is_internal_call(channel_id: uint64_t) -> bool {
    channel_id & INTERNAL_CALL_MASK != 0
}
pub const MODE_MAX_LENGTH: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
