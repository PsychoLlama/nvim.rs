#![deny(unsafe_op_in_unsafe_fn)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::api::private::helpers::{
    api_set_sctx, api_typename, cstr_to_string, find_buffer_by_handle, string_to_cstr, try_enter,
    try_leave,
};
use crate::api::private::validate::check_string_array;
use crate::autocmd::{
    apply_autocmds_group, au_get_autocmds_for_event, aucmd_del_for_event_and_group,
    aucmd_span_pattern, augroup_add, augroup_del, augroup_exists, augroup_find, augroup_name,
    aupat_get_buflocal_nr, aupat_is_buflocal, aupat_normalize_buflocal_pat, autocmd_delete_id,
    autocmd_register, do_autocmd_event, event_name2nr_str, event_nr2name,
};
use crate::buffer::do_modelines;
use crate::eval::typval::{callback_free, callback_to_string};
use crate::global_cell::GlobalCell;
use crate::lua::executor::{api_new_luaref, nlua_ref_is_function};
use crate::memory::{strequal, xfree};
use crate::strings::arena_printf;
use crate::types::AutoEvent;
use crate::types::{
    ApiDict, Arena, Array, AutoCmd, AutoCmdVec, AutoPat, BufferHandle, Callback, Error, ExArg,
    Exception, Integer, KeyDict_clear_autocmds, KeyDict_create_augroup, KeyDict_create_autocmd,
    KeyDict_exec_autocmds, KeyDict_get_autocmds, MsgList, Object, String_0, TryState, int64_t,
    kErrorTypeValidation, kObjectTypeString, size_t, uint64_t,
};
use ::libc::abort;

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
pub const AUGROUP_DEFAULT: ::core::ffi::c_int = -1;
pub const AUGROUP_ERROR: ::core::ffi::c_int = -2;
pub const AUGROUP_ALL: ::core::ffi::c_int = -3;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
static next_autocmd_id: GlobalCell<int64_t> = GlobalCell::new(1 as int64_t);
