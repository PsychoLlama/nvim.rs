#![deny(unsafe_op_in_unsafe_fn)]

use crate::src::nvim::api::extmark::ns_initialized;
use crate::src::nvim::api::private::helpers::{
    api_clear_error, api_free_array, api_set_error, api_set_sctx, arena_array, cstr_as_string,
    string_to_cstr, try_enter, try_leave,
};
use crate::src::nvim::change::inserted_bytes;
use crate::src::nvim::cursor::check_cursor;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_curbuf_later};
use crate::src::nvim::eval::funcs::find_internal_func;
use crate::src::nvim::eval::typval::tv_clear;
use crate::src::nvim::eval::userfunc::{call_func, register_luafunc};
use crate::src::nvim::event::r#loop::{loop_schedule_deferred, process_events_until};
use crate::src::nvim::event::multiqueue::multiqueue_put_event;
use crate::src::nvim::event::time::{
    time_watcher_close, time_watcher_init, time_watcher_start, time_watcher_stop,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_getln::{
    cmdpreview_get_bufnr, cmdpreview_get_ns, get_user_input, script_get,
    ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave,
};
use crate::src::nvim::garray::{
    ga_append, ga_clear, ga_concat_len, ga_concat_strings, ga_grow, ga_init,
};
use crate::src::nvim::getchar::vgetc;
use crate::src::nvim::global_cell::{GlobalCell, SharedCell};
use crate::src::nvim::keycodes::{special_to_buf, vim_unescape_ks};
use crate::src::nvim::lua::api_wrappers::nlua_add_api_functions;
use crate::src::nvim::lua::converter::{
    nlua_init_types, nlua_pop_Array, nlua_pop_Integer, nlua_pop_Object, nlua_pop_typval,
    nlua_push_Array, nlua_push_Object, nlua_push_typval,
};
use crate::src::nvim::lua::ffi::{
    lua_call, lua_checkstack, lua_close, lua_concat, lua_createtable, lua_error, lua_getfield,
    lua_getmetatable, lua_gettop, lua_insert, lua_iscfunction, lua_isnumber, lua_isstring,
    lua_newuserdata, lua_next, lua_pcall, lua_pushboolean, lua_pushcclosure, lua_pushinteger,
    lua_pushlightuserdata, lua_pushlstring, lua_pushnil, lua_pushnumber, lua_pushstring,
    lua_pushvalue, lua_rawgeti, lua_rawseti, lua_remove, lua_replace, lua_setfield,
    lua_setmetatable, lua_settop, lua_toboolean, lua_tocfunction, lua_tointeger, lua_tolstring,
    lua_touserdata, lua_type, luaL_callmeta, luaL_checkinteger, luaL_checklstring,
    luaL_checknumber, luaL_checktype, luaL_error, luaL_getmetafield, luaL_loadbuffer,
    luaL_newstate, luaL_openlibs, luaL_ref, luaL_unref, luaL_where, luaopen_luv, luv_set_loop,
};
use crate::src::nvim::lua::stdlib::nlua_state_add_stdlib;
use crate::src::nvim::lua::treesitter::nlua_treesitter_init;
use crate::src::nvim::main::{
    IObuff, cmdmod, curbuf, current_sctx, curwin, did_emsg, did_throw, e_argreq,
    e_fast_api_disabled, e_outofmem, expr_map_lock, force_abort, got_int, main_loop, mod_mask,
    nlua_disable_preload, nlua_global_refs, os_exit, p_verbose, preserve_exit, suppress_errthrow,
    textlock, time_fd, ui_event_ns_id, ui_ext_names, ui_refresh_cmdheight,
};
use crate::src::nvim::memline::{ml_get_buf, ml_get_buf_len, ml_replace};
use crate::src::nvim::memory::{
    ARENA_EMPTY, arena_finish, arena_mem_free, strequal, xcalloc, xfree, xmalloc, xmallocz,
    xmemdupz, xrealloc, xstrdup,
};
use crate::src::nvim::message::{emsg, msg_multihl, msg_putchar, semsg_multiline};
use crate::src::nvim::msgpack_rpc::channel::{rpc_send_call, rpc_send_event};
use crate::src::nvim::os::env::home_replace_save;
use crate::src::nvim::os::fileio::{file_close, file_open_stdin, file_read};
use crate::src::nvim::os::libc::{
    __assert_fail, exit, fprintf, gettext, memcpy, memset, pthread_exit, snprintf, stderr, strcmp,
    strlen,
};
use crate::src::nvim::path::fix_fname;
use crate::src::nvim::profile::{time_msg, time_pop, time_push};
use crate::src::nvim::runtime::{
    cmd_source_buffer, find_script_by_name, new_script_item, runtime_get_named_thread,
    runtime_search_path_validate, script_is_lua,
};
use crate::src::nvim::strings::{arena_printf, vim_snprintf};
use crate::src::nvim::types::ui::{kUICmdline, kUILinegrid};
use crate::src::nvim::types::{
    Arena, ArenaMem, Array, CMD_equal, CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE,
    CMOD_KEEPALT, CMOD_KEEPJUMPS, CMOD_KEEPMARKS, CMOD_KEEPPATTERNS, CMOD_LOCKMARKS,
    CMOD_NOAUTOCMD, CMOD_NOSWAPFILE, CMOD_SANDBOX, CMOD_SILENT, CMOD_UNSILENT, Error, EvalFuncDef,
    Event, FileDescriptor, HlMessage, HlMessageChunk, LuaRef, LuaRetMode, MessageData, MultiQueue,
    Object, OptInt, String_0, StringBuilder, TimeWatcher, TryState, VAR_DICT, VAR_FIXED, VAR_LIST,
    VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, buf_T, colnr_T, consumed_blk, dict_T,
    exarg_T, except_T, expand_T, funcexe_T, garray_T, handle_T, int64_t, intptr_t,
    kErrorTypeException, kErrorTypeNone, kErrorTypeValidation, kObjectTypeArray,
    kObjectTypeBoolean, kObjectTypeLuaRef, kObjectTypeNil, kObjectTypeString, linenr_T,
    lua_CFunction, lua_Integer, lua_Number, lua_State, mod_entry_T, msglist_T, nlua_ref_state_t,
    object, object_data as C2Rust_Unnamed_11, partial_T, proftime_T, ptrdiff_t, scid_T,
    scriptitem_T, sctx_T, size_t, typval_T, typval_vval_union, ucmd_T, uint8_t, uint32_t, uint64_t,
    varnumber_T,
};
use crate::src::nvim::ui::{ui_add_cb, ui_flush, ui_has, ui_remove_cb};
use crate::src::nvim::undo::u_save;
use crate::src::nvim::usercmd::{uc_mods, uc_split_args_iter};
use crate::src::nvim::window::{WSP_ABOVE, WSP_BELOW, WSP_BOT, WSP_HOR, WSP_TOP, WSP_VERT};

// The carve of the transpiled module; see each child's docs.
mod call;
mod callable;
mod error;
mod excmds;
mod exec;
mod expand;
mod luv;
mod modules;
mod print;
mod refs;
mod schedule;
mod state;
mod ucmd;
mod vim_module;

pub use self::call::*;
pub use self::callable::*;
pub use self::error::*;
pub use self::excmds::*;
pub use self::exec::*;
pub use self::expand::*;
pub(crate) use self::luv::*;
pub(crate) use self::modules::*;
pub(crate) use self::print::*;
pub use self::refs::*;
pub use self::schedule::*;
pub use self::state::*;
pub use self::ucmd::*;
pub(crate) use self::vim_module::*;
unsafe extern "C" {
    fn lua_getstack(
        L: *mut lua_State,
        level: ::core::ffi::c_int,
        ar: *mut lua_Debug,
    ) -> ::core::ffi::c_int;
    fn lua_getinfo(
        L: *mut lua_State,
        what: *const ::core::ffi::c_char,
        ar: *mut lua_Debug,
    ) -> ::core::ffi::c_int;
    fn uv_thread_self() -> uv_thread_t;
    fn uv_thread_equal(t1: *const uv_thread_t, t2: *const uv_thread_t) -> ::core::ffi::c_int;
    fn luv_set_callback(L: *mut lua_State, pcall: luv_CFpcall);
    fn luv_set_thread(L: *mut lua_State, pcall: luv_CFpcall);
    fn luv_set_cthread(L: *mut lua_State, cpcall: luv_CFcpcall);
    fn luv_set_thread_cb(acquire: luv_acquire_vm, release: luv_release_vm);
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct lua_Debug {
    pub event: ::core::ffi::c_int,
    pub name: *const ::core::ffi::c_char,
    pub namewhat: *const ::core::ffi::c_char,
    pub what: *const ::core::ffi::c_char,
    pub source: *const ::core::ffi::c_char,
    pub currentline: ::core::ffi::c_int,
    pub nups: ::core::ffi::c_int,
    pub linedefined: ::core::ffi::c_int,
    pub lastlinedefined: ::core::ffi::c_int,
    pub short_src: [::core::ffi::c_char; 60],
    pub i_ci: ::core::ffi::c_int,
}
pub type pthread_t = ::core::ffi::c_ulong;
pub type uv_thread_t = pthread_t;
pub type luv_CFpcall = Option<
    unsafe extern "C-unwind" fn(
        *mut lua_State,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
pub type luv_CFcpcall = Option<
    unsafe extern "C-unwind" fn(
        *mut lua_State,
        lua_CFunction,
        *mut ::core::ffi::c_void,
        ::core::ffi::c_int,
    ) -> ::core::ffi::c_int,
>;
pub type luv_acquire_vm = Option<unsafe extern "C-unwind" fn() -> *mut lua_State>;
pub type luv_release_vm = Option<unsafe extern "C-unwind" fn(*mut lua_State) -> ()>;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_27 = 20;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const FCERR_OTHER: C2Rust_Unnamed_31 = 6;
pub const FCERR_NONE: C2Rust_Unnamed_31 = 5;
pub type C2Rust_Unnamed_32 = ::core::ffi::c_uint;
pub const kNluaPushSpecial: C2Rust_Unnamed_32 = 1;
pub const kRetMulti: LuaRetMode = 3;
pub const kRetNilBool: LuaRetMode = 1;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModuleDef {
    pub name: *mut ::core::ffi::c_char,
    pub data: *const uint8_t,
    pub size: size_t,
}
pub const kThreadCallback: luv_err_type = 2;
pub const kThread: luv_err_type = 1;
pub type luv_err_t = luv_err_type;
pub type luv_err_type = ::core::ffi::c_uint;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_MULTRET: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const LUA_REGISTRYINDEX: ::core::ffi::c_int = -10000 as ::core::ffi::c_int;
pub const LUA_GLOBALSINDEX: ::core::ffi::c_int = -10002 as ::core::ffi::c_int;
pub const LUA_ERRMEM: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const LUA_TNIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const LUA_TBOOLEAN: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const LUA_TTABLE: ::core::ffi::c_int = 5 as ::core::ffi::c_int;
pub const LUA_TFUNCTION: ::core::ffi::c_int = 6 as ::core::ffi::c_int;
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const LUA_REFNIL: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
pub const LUVF_CALLBACK_NOEXIT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const ARRAY_DICT_INIT: Array = Array {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<Object>(),
};
pub const INTERNAL_CALL_MASK: uint64_t = (1 as ::core::ffi::c_int as uint64_t)
    << ::core::mem::size_of::<uint64_t>()
        .wrapping_mul(8 as usize)
        .wrapping_sub(1 as usize);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL.wrapping_add(1 as uint64_t);
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const EX_EXTRA: ::core::ffi::c_uint = 0x4 as ::core::ffi::c_uint;
pub const EX_NOSPC: ::core::ffi::c_uint = 0x10 as ::core::ffi::c_uint;
pub const EX_NEEDARG: ::core::ffi::c_uint = 0x80 as ::core::ffi::c_uint;
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
static in_fast_callback: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static in_script: SharedCell<bool> = SharedCell::new(false_0 != 0);
static global_lstate: GlobalCell<*mut lua_State> =
    GlobalCell::new(::core::ptr::null_mut::<lua_State>());
pub static active_lstate: GlobalCell<*mut lua_State> =
    GlobalCell::new(::core::ptr::null_mut::<lua_State>());
static require_ref: GlobalCell<LuaRef> = GlobalCell::new(LUA_REFNIL);
static main_thread: SharedCell<uv_thread_t> = SharedCell::new(0);
pub unsafe extern "C-unwind" fn get_global_lstate() -> *mut lua_State {
    return global_lstate.get();
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
