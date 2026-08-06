#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::private::helpers::{
    api_set_error, api_set_sctx, api_typename, arena_array, arena_dict, arena_string,
    arena_take_arraybuilder, cstr_as_string, find_buffer_by_handle, string_to_cstr, try_enter,
    try_leave,
};
use crate::src::nvim::api::private::validate::{
    api_err_conflict, api_err_exp, api_err_invalid, api_err_required, check_string_array,
};
use crate::src::nvim::autocmd::{
    EVENT_BUFADD, apply_autocmds_group, au_get_autocmds_for_event, aucmd_del_for_event_and_group,
    aucmd_span_pattern, augroup_add, augroup_del, augroup_exists, augroup_find, augroup_name,
    aupat_get_buflocal_nr, aupat_is_buflocal, aupat_normalize_buflocal_pat, autocmd_delete_id,
    autocmd_register, do_autocmd_event, event_name2nr_str, event_nr2name,
};
use crate::src::nvim::buffer::do_modelines;
use crate::src::nvim::eval::typval::{
    callback_free, callback_to_string, kCallbackFuncref, kCallbackLua, kCallbackNone,
};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::executor::{api_new_luaref, nlua_ref_is_function};
use crate::src::nvim::main::{curbuf, current_sctx};
use crate::src::nvim::memory::{strequal, xfree};
use crate::src::nvim::os::libc::{__assert_fail, abort, strlen};
use crate::src::nvim::strings::arena_printf;
use crate::src::nvim::types::{
    Arena, Array, ArrayBuilder, AutoCmd, AutoCmdVec, AutoPat, Buffer, Callback,
    Callback_data as C2Rust_Unnamed_5, Dict, Error, Integer, KeyDict_clear_autocmds,
    KeyDict_create_augroup, KeyDict_create_autocmd, KeyDict_exec_autocmds, KeyDict_get_autocmds,
    LuaRef, Object, String_0, TryState, auto_event, buf_T, event_T, exarg_T, except_T, int64_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray, kObjectTypeBuffer,
    kObjectTypeInteger, kObjectTypeNil, kObjectTypeString, msglist_T,
    object_data as C2Rust_Unnamed, sctx_T, size_t, uint64_t,
};

// The carve of the transpiled module; see each child's docs.
mod create;
mod exec;
mod group;
mod pattern;
mod query;

pub use self::create::*;
pub use self::exec::*;
pub use self::group::*;
pub(crate) use self::pattern::*;
pub use self::query::*;
pub const NUM_EVENTS: auto_event = 145;
pub const AUGROUP_DEFAULT: C2Rust_Unnamed_14 = -1;
pub const AUGROUP_ERROR: C2Rust_Unnamed_14 = -2;
pub const AUGROUP_ALL: C2Rust_Unnamed_14 = -3;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_int;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_clear_autocmds__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_clear_autocmds__buffer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__desc: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__buffer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__command: ::core::ffi::c_int = 7 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_autocmd__callback: ::core::ffi::c_int = 9 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__buf: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__data: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__buffer: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_exec_autocmds__modeline: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__id: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__buf: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__event: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__buffer: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_get_autocmds__pattern: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const KEYSET_OPTIDX_create_augroup__clear: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
static next_autocmd_id: GlobalCell<int64_t> = GlobalCell::new(1 as int64_t);
