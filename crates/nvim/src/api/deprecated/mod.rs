#![deny(unsafe_op_in_unsafe_fn)]

use crate::api::buffer::{nvim_buf_get_lines, nvim_buf_set_lines};
use crate::api::extmark::{nvim_buf_clear_namespace, nvim_create_namespace, parse_virt_text};
use crate::api::private::dispatch::msgpack_rpc_get_handler_for;
use crate::api::private::helpers::{
    api_clear_error, api_free_object, api_set_error, api_set_sctx, api_typename, arena_array,
    copy_object, copy_string, cstr_as_string, dict_set_var, find_buffer_by_handle,
    find_tab_by_handle, find_window_by_handle,
};
use crate::api::private::validate::{api_err_exp, api_err_invalid};
use crate::api::vimscript::exec_impl;
use crate::decoration::{clear_virttext, decor_find_virttext, kHlModeUnknown, kVPosEndOfLine};
use crate::eval::vars::get_globvar_dict;
use crate::extmark::extmark_set;
use crate::global_cell::GlobalCell;
use crate::highlight::hl_get_attr_by_id;
use crate::highlight_group::{syn_check_group, syn_get_final_id, syn_id2attr, syn_name2id};
use crate::lua::executor::nlua_exec;
use crate::main::{curbuf, current_sctx, curwin, got_int, msg_didout, msg_silent, no_wait_return};
use crate::memory::{xmalloc, xrealloc};
use crate::message::{emsg, msg, msg_end};
use crate::option::{
    find_option, get_option_value_for, get_vimoption, object_as_optval, option_has_scope,
    optval_as_object, set_option_value_for,
};
use crate::options::kOptInvalid;
use crate::pos::{MAXCOL, MAXLNUM};
use crate::types::{
    Arena, Array, Boolean, Buffer, DecorExt, DecorHighlightInline, DecorInline, DecorInlineData,
    DecorPriority, DecorVirtText, DecorVirtText_data as C2Rust_Unnamed_2, Dict, Error, Integer,
    KeyDict_empty, KeyDict_exec_opts, KeyValuePair, LuaRetMode, MsgpackRpcRequestHandler, Object,
    OptIndex, OptScope, OptVal, OptValData, OptValType, String_0, StringBuilder, Tabpage, VirtText,
    VirtTextChunk, Window, buf_T, colnr_T, int64_t, kErrorTypeNone, kErrorTypeValidation, kFalse,
    kObjectTypeArray, kObjectTypeNil, kObjectTypeString, lua_State, object,
    object_data as C2Rust_Unnamed, schar_T, sctx_T, size_t, tabpage_T, uint8_t, uint16_t, uint32_t,
    uint64_t, win_T,
};

// The carve of the transpiled module; see each child's docs.
mod bufhl;
mod eval;
mod highlight;
mod lines;
mod options;
mod vars;
mod write;

pub use self::bufhl::*;
pub use self::eval::*;
pub use self::highlight::*;
pub use self::lines::*;
pub use self::options::*;
pub use self::vars::*;
pub use self::write::*;
pub const kRetObject: LuaRetMode = 0;
pub const kOptScopeBuf: OptScope = 2;
pub const kOptScopeWin: OptScope = 1;
pub const kOptScopeGlobal: OptScope = 0;
pub const kOptValTypeNil: OptValType = -1;
pub const LINE_BUFFER_MIN_SIZE: C2Rust_Unnamed_18 = 4096;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DECOR_ID_INVALID: ::core::ffi::c_uint = UINT32_MAX;
pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000 as ::core::ffi::c_int;
pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0 as uint16_t,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0 as ::core::ffi::c_int,
    conceal_char: 0 as schar_T,
};
pub const DECOR_INLINE_INIT: DecorInline = DecorInline {
    ext: false,
    data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};
pub const NL: ::core::ffi::c_int = '\n' as ::core::ffi::c_int;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int =
    (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int) << 8 as ::core::ffi::c_int;
