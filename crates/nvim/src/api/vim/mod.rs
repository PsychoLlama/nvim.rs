#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::buffer::{api_buf_ensure_loaded, nvim_buf_del_keymap};
use crate::api::deprecated::{buffer_del_line, buffer_get_line, buffer_set_line};
use crate::api::private::converter::vim_to_object;
use crate::api::private::helpers::{
    api_metadata, api_set_error, api_set_sctx, api_typename, arena_array, arena_dict, arena_string,
    arena_take_arraybuilder, copy_array, copy_dict, copy_object, copy_string, cstr_as_string,
    dict_get_value, dict_set_var, find_buffer_by_handle, find_tab_by_handle, find_window_by_handle,
    get_default_stl_hl, parse_hl_msg, set_mark, string_to_array, try_enter, try_leave,
};
use crate::api::private::validate::{api_err_exp, api_err_invalid, api_err_required};
use crate::ascii::ascii_isdigit;
use crate::autocmd::{
    EVENT_BUFADD, EVENT_BUFNEW, apply_autocmds, block_autocmds, may_trigger_vim_suspend_resume,
    unblock_autocmds,
};
use crate::buffer::{
    buf_close_terminal, buf_get_changedtick, buflist_new, buflist_nr2name, bufref_valid, do_buffer,
    read_buffer_into, set_bufref,
};
use crate::channel::{
    channel_all_info, channel_alloc, channel_decref, channel_incref, channel_info,
    channel_internal, channel_send, find_channel,
};
use crate::context::{ctx_free, ctx_from_dict, ctx_restore, ctx_save, ctx_to_dict, kCtxAll};
use crate::cursor::get_cursor_rel_lnum;
use crate::decoration::decor_redraw_signs;
use crate::drawline::use_cursor_line_highlight;
use crate::drawscreen::{
    UPD_CLEAR, UPD_NOT_VALID, UPD_VALID, redraw_all_later, redraw_buf_later,
    redraw_buf_range_later, redraw_later, setcursor_mayforce, update_screen, win_update_cursorline,
};
use crate::eval::typval::tv_dict_find;
use crate::eval::vars::{get_globvar_dict, get_vimvar_dict, set_vim_var_nr};
use crate::ex_docmd::{changedir_func, exec_normal};
use crate::ex_eval::aborting;
use crate::fold::fold_info;
use crate::getchar::{ins_typebuf, paste_store};
use crate::global_cell::GlobalCell;
use crate::grid::{get_win_by_grid_handle, schar_cache_clear, schar_get, win_grid_alloc};
use crate::highlight::{
    dict2hlattrs, highlight_use_hlstate, hl_check_ns, hl_get_attr_by_id, hl_inspect,
    hl_ns_get_attrs, ns_hl_def, win_check_ns_hl,
};
use crate::highlight_group::{
    COLOR_NAMES, HLF_CLN, HLF_CLS, HLF_LNA, HLF_LNB, HLF_N, HLF_SC, name_to_color, ns_get_hl_defs,
    syn_check_group, syn_id2name,
};
use crate::insexpand::get_cot_flags;
use crate::keycodes::{name_to_mod_mask, replace_termcodes, vim_strsave_escape_ks};
use crate::log::LOGLVL_DBG;
use crate::lua::executor::{
    api_free_luaref, nlua_call_ref, nlua_exec, nlua_get_global_ref_count, nlua_is_deferred_safe,
};
use crate::main::{
    Columns, RedrawingDisabled, VIsual_active, arena_alloc_count, cmdpreview, cmdwin_buf, curbuf,
    current_sctx, curtab, curwin, default_grid, did_emsg, e_cmdwin, e_invchan, ex_normal_busy,
    first_tabpage, firstbuf, firstwin, g_stats, lines_left, msg_didany, msg_no_more, msg_scroll,
    msg_silent, must_redraw, need_wait_return, no_wait_return, ns_hl_fast, ns_hl_global, p_cpo,
    p_lz, pum_grid, redraw_tabline, textlock, tslua_query_parse_count, typebuf, typebuf_was_filled,
    vgetc_busy,
};
use crate::mapping::{keymap_array, modify_keymap};
use crate::mark::mark_get_global;
use crate::mbyte::{mb_string2cells, utfc_ptr2len, utfc_ptr2schar};
use crate::memline::ml_open;
use crate::memory::{arena_alloc, arena_strdup, memchrsub, strequal, xfree, xrealloc};
use crate::message::{
    do_autocmd_progress, hl_msg_free, msg_id_exists, msg_multihl, verbose_enter, verbose_leave,
    verbose_stop,
};
use crate::r#move::{changed_window_setting, update_topline, validate_cursor, win_col_off};
use crate::msgpack_rpc::channel::rpc_set_client_info;
use crate::msgpack_rpc::unpacker::unpack;
use crate::normal::reset_VIsual_and_resel;
use crate::option::{buf_copy_options, set_option_direct_for};
use crate::options::{kOptBufhidden, kOptBuftype, kOptCotFlagPopup, kOptInvalid};
use crate::optionstr::check_stl_option;
use crate::os::cshim::snprintf;
use crate::os::input::{input_blocking, input_enqueue, input_enqueue_mouse, input_enqueue_raw};
use crate::os::proc::os_proc_children;
use crate::popupmenu::{pum_ext_select_item, pum_set_info};
use crate::register::{do_put, finish_yankreg_from_object, prepare_yankreg_from_object};
use crate::runtime::{
    RuntimeOpts, do_in_runtimepath, do_source, get_lib_dir, runtime_get_named, runtime_inspect,
    script_autoload,
};
use crate::search::{BACKWARD, FORWARD};
use crate::state::get_mode;
use crate::statusline::{STL_FOLDCOL, STL_SIGNCOL, draw_tabline, win_redr_status, win_redr_winbar};
use crate::terminal::{
    terminal_alloc, terminal_buf, terminal_check_size, terminal_destroy, terminal_open,
    terminal_running, terminal_set_streamed_paste,
};
use crate::types::{
    AdditionalData, Arena, Array, ArrayBuilder, Boolean, Buffer, Channel, ChannelStreamType,
    Context, Dict, Error, Float, HlAttrs, HlMessage, Integer, KeyDict_complete_set,
    KeyDict_context, KeyDict_echo_opts, KeyDict_empty, KeyDict_eval_statusline,
    KeyDict_get_highlight, KeyDict_get_ns, KeyDict_highlight, KeyDict_keymap, KeyDict_open_term,
    KeyDict_redraw, KeyDict_runtime, KeyValuePair, LuaRef, LuaRetMode, MessageData, MessageType,
    MotionType, NS, Object, OptScope, OptVal, OptValData, OptValType, RemapValues, ScreenGrid,
    SignTextAttrs, String_0, StringBuilder, Tabpage, TerminalOptions, TryState, VV_LNUM, VV_RELNUM,
    VV_VIRTNUM, Window, bln_values, buf_T, bufref_T, dictitem_T, dobuf_action_values,
    dobuf_start_values, except_T, foldinfo_T, handle_T, int64_t, kCdScopeGlobal,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeDict, kObjectTypeInteger, kObjectTypeString, linenr_T,
    mpack_token_type_t, msg_data, msglist_T, object, object_data as C2Rust_Unnamed, pos_T,
    ptrdiff_t, schar_T, scid_T, sctx_T, size_t, statuscol_T, tabpage_T, uint8_t, uint16_t,
    uint64_t, varnumber_T, win_T, xfmark_T, yankreg_T,
};
use crate::ui::{ui_array, ui_call_screenshot, ui_flush};
use crate::window::{goto_tabpage_tp, goto_tabpage_win, win_find_tabpage};
use ::libc::{labs, memcpy, strlen};

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
pub const kMessageTypeNotification: MessageType = 2;
pub const kOptValTypeString: OptValType = 2;
pub const kOptScopeBuf: OptScope = 2;
pub const REMAP_NONE: RemapValues = -1;
pub const REMAP_YES: RemapValues = 0;
pub const REPTERM_NO_SPECIAL: C2Rust_Unnamed_36 = 4;
pub const REPTERM_DO_LT: C2Rust_Unnamed_36 = 2;
pub const REPTERM_FROM_PART: C2Rust_Unnamed_36 = 1;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetObject: LuaRetMode = 0;
#[derive(Copy, Clone)]
pub struct RuntimeCookie {
    pub rv: ArrayBuilder,
    pub arena: *mut Arena,
}
pub const DOSO_NONE: C2Rust_Unnamed_40 = 0;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_NEW: bln_values = 8;
pub const BLN_NOOPT: bln_values = 16;
pub const BCO_NOHELP: C2Rust_Unnamed_37 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_37 = 1;
pub const MPACK_TOKEN_SINT: mpack_token_type_t = 4;
pub const kChannelStreamInternal: ChannelStreamType = 4;
pub const kMTCharWise: MotionType = 0;
pub const kCtxFuncs: C2Rust_Unnamed_33 = 32;
pub const kCtxSFuncs: C2Rust_Unnamed_33 = 16;
pub const kCtxGVars: C2Rust_Unnamed_33 = 8;
pub const kCtxBufs: C2Rust_Unnamed_33 = 4;
pub const kCtxJumps: C2Rust_Unnamed_33 = 2;
pub const kCtxRegs: C2Rust_Unnamed_33 = 1;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_36 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_37 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_38 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_40 = ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const MAX_SCHAR_SIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
pub const STRING_INIT: String_0 = String_0 {
    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    size: 0 as size_t,
};
/// The top bit of a channel id: an API call made by the editor itself
/// rather than by a client.
pub const INTERNAL_CALL_MASK: uint64_t = 1 << (::core::mem::size_of::<uint64_t>() * 8 - 1);
#[inline(always)]
fn is_internal_call(channel_id: uint64_t) -> bool {
    return channel_id & INTERNAL_CALL_MASK != 0;
}
pub const KEYSET_OPTIDX_context__types: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__fillchar: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__maxwidth: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_eval_statusline__use_statuscol_lnum: ::core::ffi::c_int =
    7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__url: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_highlight__update: ::core::ffi::c_int = 13 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_ns__winid: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_open_term__on_input: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_open_term__force_crlf: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_complete_set__info: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__win: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__flush: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__range: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_redraw__valid: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const NULL_STRING: String_0 = STRING_INIT;
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const CONTEXT_INIT: Context = Context {
    regs: STRING_INIT,
    jumps: STRING_INIT,
    bufs: STRING_INIT,
    gvars: STRING_INIT,
    funcs: Array {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<Object>(),
    },
};
pub const MODE_MAX_LENGTH: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
