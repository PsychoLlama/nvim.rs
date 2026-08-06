#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::cjson::lua_cjson::lua_cjson_new;
use crate::src::mpack::lmpack::luaopen_mpack;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, dict_check_writable, find_buffer_by_handle, find_tab_by_handle,
    find_window_by_handle, try_enter, try_leave,
};
use crate::src::nvim::autocmd::{aucmd_prepbuf, aucmd_restbuf};
use crate::src::nvim::eval::typval::tv_dict_is_watched;
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_find, tv_dict_item_alloc_len, tv_dict_item_remove,
    tv_dict_watcher_notify,
};
use crate::src::nvim::eval::vars::{before_set_vvar, get_globvar_dict, get_vimvar_dict};
use crate::src::nvim::eval::window::{win_execute_after, win_execute_before};
use crate::src::nvim::ex_docmd::{apply_cmdmod, undo_cmdmod};
use crate::src::nvim::ex_eval::aborting;
use crate::src::nvim::fold::foldUpdate;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::lua::base64::luaopen_base64;
use crate::src::nvim::lua::converter::{nlua_pop_typval, nlua_push_typval};
use crate::src::nvim::lua::ffi::{
    lua_concat, lua_createtable, lua_error, lua_getfield, lua_gettop, lua_newuserdata, lua_next,
    lua_pcall, lua_pushcclosure, lua_pushinteger, lua_pushlstring, lua_pushnil, lua_pushnumber,
    lua_pushstring, lua_pushvalue, lua_pushvfstring, lua_rawseti, lua_setfield, lua_setmetatable,
    lua_settop, lua_toboolean, lua_tolstring, lua_type, luaL_argerror, luaL_checkinteger,
    luaL_checklstring, luaL_checkudata, luaL_error, luaL_newmetatable, luaL_register, luaL_where,
};
use crate::src::nvim::lua::spell::luaopen_spell;
use crate::src::nvim::lua::xdiff::nlua_xdl_diff;
use crate::src::nvim::main::{buffer_handles, cmdmod, curbuf, g_min_log_level, window_handles};
use crate::src::nvim::map::mh_get_int;
use crate::src::nvim::mbyte::{
    convert_setup, convert_setup_ext, enc_canonize, enc_skip, mb_utf_index_to_bytes, mb_utflen,
    string_convert, utf_cp_bounds_len, utf_ptr2len_len,
};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len};
use crate::src::nvim::memory::{strequal, xfree};
use crate::src::nvim::os::libc::{__assert_fail, memchr, memset, strcasecmp};
use crate::src::nvim::regexp::{vim_regcomp, vim_regexec, vim_regfree};
use crate::src::nvim::runtime::script_autoload;
use crate::src::nvim::types::{
    Buffer, CMOD_ERRSILENT, CMOD_HIDE, CMOD_KEEPALT, CMOD_KEEPJUMPS, CMOD_KEEPMARKS,
    CMOD_KEEPPATTERNS, CMOD_LOCKMARKS, CMOD_NOAUTOCMD, CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT,
    Error, Map_int_ptr_t, String_0, Tabpage, TryState, VAR_UNKNOWN, VAR_UNLOCKED, Window,
    aco_save_T, buf_T, cmdmod_T, colnr_T, dict_T, dictitem_T, except_T, handle_T, intptr_t,
    kErrorTypeNone, linenr_T, lua_Integer, lua_Number, lua_State, luaL_Reg, msglist_T, pos_T,
    ptr_t, ptrdiff_t, regmatch_T, regprog_T, size_t, ssize_t, switchwin_T, tabpage_T, typval_T,
    typval_vval_union, uint32_t, vimconv_T, win_T, win_execute_T,
};
use crate::src::nvim::window::win_find_tabpage;

// The carve of the transpiled module; see each child's docs.
mod regex;
mod register;
mod strings;
mod vars;
mod with;

pub use self::regex::*;
pub use self::register::*;
pub use self::strings::*;
pub use self::vars::*;
pub(crate) use self::with::*;
pub type C2Rust_Unnamed_13 = ::core::ffi::c_uint;
pub const CONV_NONE: C2Rust_Unnamed_13 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TSTRING: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C-unwind" fn map_get_int_ptr_t(
    mut map: *mut Map_int_ptr_t,
    mut key: ::core::ffi::c_int,
) -> ptr_t {
    unsafe {
        let mut k: uint32_t = mh_get_int(&raw mut (*map).set, key);
        return if k == MH_TOMBSTONE as uint32_t {
            value_init_ptr_t.get()
        } else {
            *(*map).values.offset(k as isize)
        };
    }
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
