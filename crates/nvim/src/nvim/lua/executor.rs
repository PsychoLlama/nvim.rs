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
    Arena, ArenaMem, Array, CMD_index, CMOD_BROWSE, CMOD_CONFIRM, CMOD_ERRSILENT, CMOD_HIDE,
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
pub const CMD_equal: CMD_index = 552;
pub const CMD_snext: CMD_index = 414;
pub const CMD_drop: CMD_index = 130;
pub const CMD_arglocal: CMD_index = 14;
pub const CMD_argglobal: CMD_index = 13;
pub const CMD_argdo: CMD_index = 10;
pub const CMD_args: CMD_index = 7;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
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
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
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
unsafe extern "C-unwind" fn nlua_get_error(
    mut lstate: *mut lua_State,
    mut len: *mut size_t,
) -> *const ::core::ffi::c_char {
    if luaL_getmetafield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION
            && luaL_callmeta(
                lstate,
                -2 as ::core::ffi::c_int,
                b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0
        {
            lua_replace(lstate, -3 as ::core::ffi::c_int);
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    return lua_tolstring(lstate, -1 as ::core::ffi::c_int, len);
}
pub unsafe extern "C-unwind" fn nlua_error(
    lstate: *mut lua_State,
    msg: *const ::core::ffi::c_char,
) {
    let mut len: size_t = 0;
    let mut str: *const ::core::ffi::c_char = nlua_get_error(lstate, &raw mut len);
    if in_script.get() {
        fprintf(stderr, msg, len as ::core::ffi::c_int, str);
        fprintf(stderr, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        semsg_multiline(
            b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
            msg,
            len as ::core::ffi::c_int,
            str,
        );
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
pub unsafe extern "C-unwind" fn nlua_pcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresults: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"debug\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"traceback\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_remove(lstate, -2 as ::core::ffi::c_int);
    lua_insert(lstate, -2 as ::core::ffi::c_int - nargs);
    let mut pre_top: ::core::ffi::c_int = lua_gettop(lstate);
    let mut status: ::core::ffi::c_int =
        lua_pcall(lstate, nargs, nresults, -2 as ::core::ffi::c_int - nargs);
    if status != 0 {
        lua_remove(lstate, -2 as ::core::ffi::c_int);
    } else {
        if nresults == LUA_MULTRET {
            nresults = lua_gettop(lstate) - (pre_top - nargs - 1 as ::core::ffi::c_int);
        }
        lua_remove(lstate, -1 as ::core::ffi::c_int - nresults);
    }
    return status;
}
unsafe extern "C" fn nlua_luv_error_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut error: *mut ::core::ffi::c_char =
        *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
    let mut type_0: luv_err_t = (*argv.offset(1 as ::core::ffi::c_int as isize)).expose_provenance()
        as intptr_t as luv_err_t;
    match type_0 as ::core::ffi::c_uint {
        0 => {
            semsg_multiline(
                b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Lua callback:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                error,
            );
        }
        1 => {
            semsg_multiline(
                b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Luv thread:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                error,
            );
        }
        2 => {
            semsg_multiline(
                b"lua_error\0".as_ptr() as *const ::core::ffi::c_char,
                b"Luv callback, thread:\n%s\0".as_ptr() as *const ::core::ffi::c_char,
                error,
            );
        }
        _ => {}
    }
    xfree(error as *mut ::core::ffi::c_void);
}
unsafe extern "C-unwind" fn nlua_fast_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    (*in_fast_callback.ptr()) += 1;
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    let mut status: ::core::ffi::c_int = nlua_pcall(lstate, nargs, nresult);
    if status != 0 {
        if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
            preserve_exit(&raw const e_outofmem as *const ::core::ffi::c_char);
        }
        let mut len: size_t = 0;
        let mut error: *const ::core::ffi::c_char = nlua_get_error(lstate, &raw mut len);
        multiqueue_put_event(
            (*main_loop.ptr()).events,
            Event {
                handler: Some(
                    nlua_luv_error_event
                        as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    (if !error.is_null() {
                        xstrdup(error)
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    }) as *mut ::core::ffi::c_void,
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        retval = -status;
    } else {
        if nresult == LUA_MULTRET {
            nresult = lua_gettop(lstate) - top + nargs + 1 as ::core::ffi::c_int;
        }
        retval = nresult;
    }
    (*in_fast_callback.ptr()) -= 1;
    return retval;
}
unsafe extern "C-unwind" fn nlua_luv_thread_cb_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, true_0 != 0);
}
unsafe extern "C-unwind" fn nlua_luv_thread_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    return nlua_luv_thread_common_cfpcall(lstate, nargs, nresult, flags, false_0 != 0);
}
unsafe extern "C-unwind" fn nlua_luv_thread_cfcpcall(
    mut lstate: *mut lua_State,
    mut func: lua_CFunction,
    mut ud: *mut ::core::ffi::c_void,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    lua_pushcclosure(lstate, func, 0 as ::core::ffi::c_int);
    lua_pushlightuserdata(lstate, ud);
    let mut retval: ::core::ffi::c_int = nlua_luv_thread_cfpcall(
        lstate,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        flags,
    );
    return retval;
}
unsafe extern "C-unwind" fn nlua_luv_thread_common_cfpcall(
    mut lstate: *mut lua_State,
    mut nargs: ::core::ffi::c_int,
    mut nresult: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut is_callback: bool,
) -> ::core::ffi::c_int {
    let mut retval: ::core::ffi::c_int = 0;
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    let mut status: ::core::ffi::c_int = lua_pcall(lstate, nargs, nresult, 0 as ::core::ffi::c_int);
    if status != 0 {
        if status == LUA_ERRMEM && flags & LUVF_CALLBACK_NOEXIT == 0 {
            fprintf(
                stderr,
                b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
                &raw const e_outofmem as *const ::core::ffi::c_char,
            );
            lua_close(lstate);
            pthread_exit(::core::ptr::null_mut::<::core::ffi::c_void>());
        }
        let mut error: *const ::core::ffi::c_char = lua_tolstring(
            lstate,
            -1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<size_t>(),
        );
        loop_schedule_deferred(
            main_loop.ptr(),
            Event {
                handler: Some(
                    nlua_luv_error_event
                        as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                ),
                argv: [
                    (if !error.is_null() {
                        xstrdup(error)
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    }) as *mut ::core::ffi::c_void,
                    ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                        (if is_callback as ::core::ffi::c_int != 0 {
                            kThreadCallback as ::core::ffi::c_int
                        } else {
                            kThread as ::core::ffi::c_int
                        }) as intptr_t as usize,
                    ),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ],
            },
        );
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        retval = -status;
    } else {
        if nresult == LUA_MULTRET {
            nresult = lua_gettop(lstate) - top + nargs + 1 as ::core::ffi::c_int;
        }
        retval = nresult;
    }
    return retval;
}
unsafe extern "C-unwind" fn nlua_thr_api_nvim__get_runtime(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    if lua_gettop(lstate) != 3 as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"Expected 3 arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TTABLE);
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"is_lua\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !(lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TBOOLEAN) {
        return luaL_error(
            lstate,
            b"is_lua is not a boolean\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut is_lua: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TBOOLEAN);
    let mut all: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let pat: Array = nlua_pop_Array(lstate, ::core::ptr::null_mut::<Arena>(), &raw mut err);
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        luaL_where(lstate, 1 as ::core::ffi::c_int);
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        lua_concat(lstate, 2 as ::core::ffi::c_int);
        return lua_error(lstate);
    }
    let mut ret: Array = runtime_get_named_thread(is_lua, pat, all);
    nlua_push_Array(lstate, ret, kNluaPushSpecial as ::core::ffi::c_int);
    api_free_array(ret);
    api_free_array(pat);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_init_argv(
    L: *mut lua_State,
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut lua_arg0: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    lua_createtable(L, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    if lua_arg0 > 0 as ::core::ffi::c_int {
        lua_pushstring(
            L,
            *argv.offset((lua_arg0 - 1 as ::core::ffi::c_int) as isize),
        );
        lua_rawseti(L, -2 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        while i + lua_arg0 < argc {
            lua_pushstring(L, *argv.offset((i + lua_arg0) as isize));
            lua_rawseti(L, -2 as ::core::ffi::c_int, i + 1 as ::core::ffi::c_int);
            i += 1;
        }
    }
    lua_setfield(
        L,
        LUA_GLOBALSINDEX,
        b"arg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return i;
}
unsafe extern "C" fn nlua_schedule_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut cb: LuaRef =
        (*argv.offset(0 as ::core::ffi::c_int as isize)).expose_provenance() as ptrdiff_t as LuaRef;
    let mut ns_id: uint32_t = (*argv.offset(1 as ::core::ffi::c_int as isize)).expose_provenance()
        as ptrdiff_t as uint32_t;
    let lstate: *mut lua_State = global_lstate.get();
    nlua_pushref(lstate, cb);
    nlua_unref_global(lstate, cb);
    let mut save_expr_map_lock: ::core::ffi::c_int = expr_map_lock.get();
    let mut save_textlock: ::core::ffi::c_int = textlock.get();
    expr_map_lock.set(if ns_id > 0 as uint32_t {
        0 as ::core::ffi::c_int
    } else {
        expr_map_lock.get()
    });
    textlock.set(if ns_id > 0 as uint32_t {
        0 as ::core::ffi::c_int
    } else {
        textlock.get()
    });
    if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        nlua_error(
            lstate,
            gettext(b"vim.schedule callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        ui_remove_cb(ns_id, true_0 != 0);
    }
    expr_map_lock.set(save_expr_map_lock);
    textlock.set(save_textlock);
}
unsafe extern "C-unwind" fn nlua_schedule(lstate: *mut lua_State) -> ::core::ffi::c_int {
    if lua_type(lstate, 1 as ::core::ffi::c_int) != LUA_TFUNCTION {
        lua_pushlstring(
            lstate,
            b"vim.schedule: expected function\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 32]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_sub(1 as size_t),
        );
        return lua_error(lstate);
    }
    lua_pushnil(lstate);
    if (*main_loop.ptr()).closing {
        lua_pushlstring(
            lstate,
            b"main loop is closing\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 21]>()
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                .wrapping_sub(1 as size_t),
        );
        return 2 as ::core::ffi::c_int;
    }
    let mut cb: LuaRef = nlua_ref_global(lstate, 1 as ::core::ffi::c_int);
    multiqueue_put_event(
        (*main_loop.ptr()).events,
        Event {
            handler: Some(
                nlua_schedule_event as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
            ),
            argv: [
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    cb as ptrdiff_t as usize,
                ),
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    ui_event_ns_id.get() as ptrdiff_t as usize,
                ),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
                ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ],
        },
    );
    lua_pushnil(lstate);
    return 2 as ::core::ffi::c_int;
}
unsafe extern "C" fn dummy_timer_due_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    if (*main_loop.ptr()).closing {
        time_watcher_stop(tw);
        time_watcher_close(
            tw,
            Some(
                dummy_timer_close_cb
                    as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
            ),
        );
    }
}
unsafe extern "C" fn dummy_timer_close_cb(
    mut tw: *mut TimeWatcher,
    mut _data: *mut ::core::ffi::c_void,
) {
    xfree(tw as *mut ::core::ffi::c_void);
}
unsafe extern "C-unwind" fn nlua_wait_condition(
    mut lstate: *mut lua_State,
    mut status: *mut ::core::ffi::c_int,
    mut callback_result: *mut bool,
    mut nresults: *mut ::core::ffi::c_int,
) -> bool {
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    lua_pushvalue(lstate, 2 as ::core::ffi::c_int);
    *status = nlua_pcall(lstate, 0 as ::core::ffi::c_int, LUA_MULTRET);
    if *status != 0 {
        return true_0 != 0;
    }
    *nresults = lua_gettop(lstate) - top;
    if *nresults == 0 as ::core::ffi::c_int {
        *callback_result = false_0 != 0;
        return false_0 != 0;
    }
    *callback_result = lua_toboolean(lstate, top + 1 as ::core::ffi::c_int) != 0;
    if !*callback_result {
        lua_settop(lstate, top);
        return false_0 != 0;
    }
    lua_remove(lstate, top + 1 as ::core::ffi::c_int);
    *nresults -= 1;
    return true_0 != 0;
}
unsafe extern "C-unwind" fn nlua_wait(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    if in_fast_callback.get() != 0 {
        return luaL_error(
            lstate,
            &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
            b"vim.wait\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut timeout_number: ::core::ffi::c_double =
        luaL_checknumber(lstate, 1 as ::core::ffi::c_int);
    if timeout_number < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
        return luaL_error(
            lstate,
            b"timeout must be >= 0\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut timeout: int64_t = if timeout_number.is_nan() as i32 != 0
        || timeout_number > INT64_MAX as ::core::ffi::c_double
    {
        INT64_MAX as int64_t
    } else {
        timeout_number as int64_t
    };
    let mut lua_top: ::core::ffi::c_int = lua_gettop(lstate);
    let mut is_function: bool = false_0 != 0;
    if lua_top >= 2 as ::core::ffi::c_int
        && !(lua_type(lstate, 2 as ::core::ffi::c_int) == LUA_TNIL)
    {
        is_function = lua_type(lstate, 2 as ::core::ffi::c_int) == LUA_TFUNCTION;
        if !is_function
            && luaL_getmetafield(
                lstate,
                2 as ::core::ffi::c_int,
                b"__call\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0 as ::core::ffi::c_int
        {
            is_function = lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION;
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
        if !is_function {
            lua_pushlstring(
                lstate,
                b"vim.wait: callback must be callable\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 36]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_sub(1 as size_t),
            );
            return lua_error(lstate);
        }
    }
    let mut interval: intptr_t = 200 as ::core::ffi::c_int as intptr_t;
    if lua_top >= 3 as ::core::ffi::c_int
        && !(lua_type(lstate, 3 as ::core::ffi::c_int) == LUA_TNIL)
    {
        interval = luaL_checkinteger(lstate, 3 as ::core::ffi::c_int) as intptr_t;
        if interval < 0 as intptr_t {
            return luaL_error(
                lstate,
                b"interval must be >= 0\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    }
    let mut fast_only: bool = false_0 != 0;
    if lua_top >= 4 as ::core::ffi::c_int {
        fast_only = lua_toboolean(lstate, 4 as ::core::ffi::c_int) != 0;
    }
    let mut loop_events: *mut MultiQueue = if fast_only as ::core::ffi::c_int != 0 {
        (*main_loop.ptr()).fast_events
    } else {
        (*main_loop.ptr()).events
    };
    let mut tw: *mut TimeWatcher =
        xmalloc(::core::mem::size_of::<TimeWatcher>()) as *mut TimeWatcher;
    time_watcher_init(main_loop.ptr(), tw, NULL);
    (*tw).events = ::core::ptr::null_mut::<MultiQueue>();
    time_watcher_start(
        tw,
        Some(
            dummy_timer_due_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
        interval as uint64_t,
        interval as uint64_t,
    );
    let mut pcall_status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut callback_result: bool = false_0 != 0;
    let mut nresults: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    ui_flush();
    process_events_until(main_loop.ptr(), loop_events, timeout, || {
        got_int.get()
            || is_function
                && nlua_wait_condition(
                    lstate,
                    &raw mut pcall_status,
                    &raw mut callback_result,
                    &raw mut nresults,
                )
    });
    time_watcher_stop(tw);
    time_watcher_close(
        tw,
        Some(
            dummy_timer_close_cb
                as unsafe extern "C" fn(*mut TimeWatcher, *mut ::core::ffi::c_void) -> (),
        ),
    );
    if pcall_status != 0 {
        return lua_error(lstate);
    } else if callback_result {
        lua_pushboolean(lstate, 1 as ::core::ffi::c_int);
        if nresults == 0 as ::core::ffi::c_int {
            lua_pushnil(lstate);
            nresults = 1 as ::core::ffi::c_int;
        } else {
            lua_insert(lstate, -1 as ::core::ffi::c_int - nresults);
        }
        return nresults + 1 as ::core::ffi::c_int;
    } else if got_int.get() {
        got_int.set(false_0 != 0);
        vgetc();
        lua_pushboolean(lstate, 0 as ::core::ffi::c_int);
        lua_pushinteger(lstate, -2 as lua_Integer);
        return 2 as ::core::ffi::c_int;
    } else {
        lua_pushboolean(lstate, 0 as ::core::ffi::c_int);
        lua_pushinteger(lstate, -1 as lua_Integer);
        return 2 as ::core::ffi::c_int;
    };
}
unsafe extern "C-unwind" fn nlua_new_ref_state(
    mut lstate: *mut lua_State,
    mut is_thread: bool,
) -> *mut nlua_ref_state_t {
    let mut ref_state: *mut nlua_ref_state_t =
        lua_newuserdata(lstate, ::core::mem::size_of::<nlua_ref_state_t>())
            as *mut nlua_ref_state_t;
    memset(
        ref_state as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<nlua_ref_state_t>(),
    );
    (*ref_state).nil_ref = LUA_NOREF as LuaRef;
    (*ref_state).empty_dict_ref = LUA_NOREF as LuaRef;
    if !is_thread {
        nlua_global_refs.set(ref_state);
    }
    return ref_state;
}
unsafe extern "C-unwind" fn nlua_get_ref_state(
    mut lstate: *mut lua_State,
) -> *mut nlua_ref_state_t {
    lua_getfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nlua.ref_state\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut ref_state: *mut nlua_ref_state_t =
        lua_touserdata(lstate, -1 as ::core::ffi::c_int) as *mut nlua_ref_state_t;
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return ref_state;
}
pub unsafe extern "C-unwind" fn nlua_get_nil_ref(mut lstate: *mut lua_State) -> LuaRef {
    let mut ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
    return (*ref_state).nil_ref;
}
pub unsafe extern "C-unwind" fn nlua_get_empty_dict_ref(mut lstate: *mut lua_State) -> LuaRef {
    let mut ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
    return (*ref_state).empty_dict_ref;
}
pub unsafe extern "C-unwind" fn nlua_get_global_ref_count() -> ::core::ffi::c_int {
    return (*nlua_global_refs.get()).ref_count;
}
unsafe extern "C-unwind" fn nlua_common_vim_init(
    mut lstate: *mut lua_State,
    mut is_thread: bool,
    mut is_standalone: bool,
) {
    let mut ref_state: *mut nlua_ref_state_t = nlua_new_ref_state(lstate, is_thread);
    lua_setfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nlua.ref_state\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(lstate, is_thread as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_is_thread as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"is_thread\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_newuserdata(lstate, 0 as size_t);
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushcclosure(
        lstate,
        Some(
            nlua_nil_tostring as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setmetatable(lstate, -2 as ::core::ffi::c_int);
    (*ref_state).nil_ref = nlua_ref(lstate, ref_state, -1 as ::core::ffi::c_int);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"mpack.NIL\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"NIL\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushcclosure(
        lstate,
        Some(
            nlua_empty_dict_tostring
                as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"__tostring\0".as_ptr() as *const ::core::ffi::c_char,
    );
    (*ref_state).empty_dict_ref = nlua_ref(lstate, ref_state, -1 as ::core::ffi::c_int);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"mpack.empty_dict\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"_empty_dict_mt\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !is_standalone {
        if is_thread {
            luv_set_callback(
                lstate,
                Some(
                    nlua_luv_thread_cb_cfpcall
                        as unsafe extern "C-unwind" fn(
                            *mut lua_State,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                        )
                            -> ::core::ffi::c_int,
                ),
            );
            luv_set_thread(
                lstate,
                Some(
                    nlua_luv_thread_cfpcall
                        as unsafe extern "C-unwind" fn(
                            *mut lua_State,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                        )
                            -> ::core::ffi::c_int,
                ),
            );
            luv_set_cthread(
                lstate,
                Some(
                    nlua_luv_thread_cfcpcall
                        as unsafe extern "C-unwind" fn(
                            *mut lua_State,
                            lua_CFunction,
                            *mut ::core::ffi::c_void,
                            ::core::ffi::c_int,
                        )
                            -> ::core::ffi::c_int,
                ),
            );
        } else {
            luv_set_loop(lstate, &raw mut (*main_loop.ptr()).uv);
            luv_set_callback(
                lstate,
                Some(
                    nlua_fast_cfpcall
                        as unsafe extern "C-unwind" fn(
                            *mut lua_State,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                            ::core::ffi::c_int,
                        )
                            -> ::core::ffi::c_int,
                ),
            );
        }
    }
    luaopen_luv(lstate);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -3 as ::core::ffi::c_int,
        b"uv\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -3 as ::core::ffi::c_int,
        b"loop\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushvalue(lstate, -3 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"luv\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
}
unsafe extern "C-unwind" fn nlua_module_preloader(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    let mut i: size_t = lua_tointeger(lstate, LUA_GLOBALSINDEX - 1 as ::core::ffi::c_int) as size_t;
    let mut def: ModuleDef = (*builtin_modules.ptr())[i as usize];
    if luaL_loadbuffer(
        lstate,
        def.data as *const ::core::ffi::c_char,
        def.size.wrapping_sub(1 as size_t),
        ::core::ptr::null::<::core::ffi::c_char>(),
    ) != 0
    {
        return lua_error(lstate);
    }
    lua_call(lstate, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_init_packages(
    mut lstate: *mut lua_State,
    mut is_standalone: bool,
) -> bool {
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"preload\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut i: size_t = 0 as size_t;
    while i < ::core::mem::size_of::<[ModuleDef; 21]>()
        .wrapping_div(::core::mem::size_of::<ModuleDef>())
        .wrapping_div(
            (::core::mem::size_of::<[ModuleDef; 21]>()
                .wrapping_rem(::core::mem::size_of::<ModuleDef>())
                == 0) as ::core::ffi::c_int as usize,
        )
    {
        let mut def: ModuleDef = (*builtin_modules.ptr())[i as usize];
        lua_pushinteger(lstate, i as lua_Integer);
        lua_pushcclosure(
            lstate,
            Some(
                nlua_module_preloader
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            1 as ::core::ffi::c_int,
        );
        lua_setfield(lstate, -2 as ::core::ffi::c_int, def.name);
        if nlua_disable_preload.get() as ::core::ffi::c_int != 0
            && !is_standalone
            && strequal(
                def.name,
                b"vim.inspect\0".as_ptr() as *const ::core::ffi::c_char,
            ) as ::core::ffi::c_int
                != 0
        {
            break;
        }
        i = i.wrapping_add(1);
    }
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"require\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushstring(
        lstate,
        b"vim._init_packages\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if nlua_pcall(lstate, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            lua_tolstring(
                lstate,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ),
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
unsafe extern "C-unwind" fn nlua_ui_attach(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut ns_id: uint32_t = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as uint32_t;
    if !ns_initialized(ns_id) {
        return luaL_error(
            lstate,
            b"invalid ns_id\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if !(lua_type(lstate, 2 as ::core::ffi::c_int) == LUA_TTABLE) {
        return luaL_error(
            lstate,
            b"opts must be a table\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    if !(lua_type(lstate, 3 as ::core::ffi::c_int) == LUA_TFUNCTION) {
        return luaL_error(
            lstate,
            b"callback must be a Lua function\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut ext_widgets: [bool; 5] = [false_0 != 0, false, false, false, false];
    let mut tbl_has_true_val: bool = false_0 != 0;
    lua_pushvalue(lstate, 2 as ::core::ffi::c_int);
    lua_pushnil(lstate);
    while lua_next(lstate, -2 as ::core::ffi::c_int) != 0 {
        let mut len: size_t = 0;
        let mut s: *const ::core::ffi::c_char =
            lua_tolstring(lstate, -2 as ::core::ffi::c_int, &raw mut len);
        let mut val: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        '_ok: {
            if strequal(s, b"set_cmdheight\0".as_ptr() as *const ::core::ffi::c_char) {
                ui_refresh_cmdheight.set(val);
            } else {
                let mut i: size_t = 0 as size_t;
                while i < kUILinegrid as ::core::ffi::c_int as size_t {
                    if strequal(
                        s,
                        *(ui_ext_names.ptr() as *mut *const ::core::ffi::c_char).offset(i as isize),
                    ) {
                        if val {
                            tbl_has_true_val = true_0 != 0;
                        }
                        ext_widgets[i as usize] = val;
                        break '_ok;
                    } else {
                        i = i.wrapping_add(1);
                    }
                }
                return luaL_error(
                    lstate,
                    b"Unexpected key: %s\0".as_ptr() as *const ::core::ffi::c_char,
                    s,
                );
            }
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    if !tbl_has_true_val {
        return luaL_error(
            lstate,
            b"opts table must contain at least one 'true' ext_widget\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
    }
    let mut ui_event_cb: LuaRef = nlua_ref_global(lstate, 3 as ::core::ffi::c_int);
    ui_add_cb(ns_id, ui_event_cb, &raw mut ext_widgets as *mut bool);
    ui_refresh_cmdheight.set(true_0 != 0);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_ui_detach(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut ns_id: uint32_t = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as uint32_t;
    if !ns_initialized(ns_id) {
        return luaL_error(
            lstate,
            b"invalid ns_id\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    ui_remove_cb(ns_id, false_0 != 0);
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_state_init(lstate: *mut lua_State) -> bool {
    lua_pushcclosure(
        lstate,
        Some(nlua_print as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"print\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"debug\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_debug as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"debug\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    nlua_add_api_functions(lstate);
    nlua_init_types(lstate);
    lua_pushcclosure(
        lstate,
        Some(nlua_schedule as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"schedule\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(
            nlua_in_fast_event as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
        ),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"in_fast_event\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_call as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"call\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_rpcrequest as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"rpcrequest\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_rpcnotify as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"rpcnotify\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_wait as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"wait\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_ui_attach as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"ui_attach\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushcclosure(
        lstate,
        Some(nlua_ui_detach as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
        0 as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"ui_detach\0".as_ptr() as *const ::core::ffi::c_char,
    );
    nlua_common_vim_init(lstate, false_0 != 0, false_0 != 0);
    if !(*time_fd.ptr()).is_null() {
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"require\0".as_ptr() as *const ::core::ffi::c_char,
        );
        require_ref.set(nlua_ref_global(lstate, -1 as ::core::ffi::c_int));
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_pushcclosure(
            lstate,
            Some(nlua_require as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"require\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    nlua_treesitter_init(lstate);
    nlua_state_add_stdlib(lstate, false_0 != 0);
    lua_setfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"vim\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !nlua_init_packages(lstate, false_0 != 0) {
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C-unwind" fn nlua_init(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut lua_arg0: ::core::ffi::c_int,
) {
    let mut lstate: *mut lua_State = luaL_newstate();
    if lstate.is_null() {
        fprintf(
            stderr,
            gettext(b"E970: Failed to initialize Lua interpreter\n\0".as_ptr()
                as *const ::core::ffi::c_char),
        );
        os_exit(1 as ::core::ffi::c_int);
    }
    luaL_openlibs(lstate);
    if !nlua_state_init(lstate) {
        fprintf(
            stderr,
            gettext(
                b"E970: Failed to initialize builtin Lua modules\n\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ),
        );
        os_exit(1 as ::core::ffi::c_int);
    }
    luv_set_thread_cb(
        Some(nlua_thread_acquire_vm as unsafe extern "C-unwind" fn() -> *mut lua_State),
        Some(nlua_common_free_all_mem as unsafe extern "C-unwind" fn(*mut lua_State) -> ()),
    );
    global_lstate.set(lstate);
    active_lstate.set(lstate);
    main_thread.set(uv_thread_self());
    nlua_init_argv(lstate, argv, argc, lua_arg0);
}
unsafe extern "C-unwind" fn nlua_thread_acquire_vm() -> *mut lua_State {
    return nlua_init_state(true_0 != 0);
}
pub unsafe extern "C-unwind" fn nlua_run_script(
    mut argv: *mut *mut ::core::ffi::c_char,
    mut argc: ::core::ffi::c_int,
    mut lua_arg0: ::core::ffi::c_int,
) -> ! {
    in_script.set(true_0 != 0);
    global_lstate.set(nlua_init_state(false_0 != 0));
    luv_set_thread_cb(
        Some(nlua_thread_acquire_vm as unsafe extern "C-unwind" fn() -> *mut lua_State),
        Some(nlua_common_free_all_mem as unsafe extern "C-unwind" fn(*mut lua_State) -> ()),
    );
    nlua_init_argv(global_lstate.get(), argv, argc, lua_arg0);
    let mut lua_ok: bool =
        nlua_exec_file(*argv.offset((lua_arg0 - 1 as ::core::ffi::c_int) as isize));
    exit(if lua_ok as ::core::ffi::c_int != 0 {
        0 as ::core::ffi::c_int
    } else {
        1 as ::core::ffi::c_int
    });
}
unsafe extern "C-unwind" fn nlua_init_state(mut thread: bool) -> *mut lua_State {
    let self_0: uv_thread_t = uv_thread_self();
    if !in_script.get() && uv_thread_equal(main_thread.ptr(), &raw const self_0) != 0 {
        runtime_search_path_validate();
    }
    let mut lstate: *mut lua_State = luaL_newstate();
    luaL_openlibs(lstate);
    if !in_script.get() {
        lua_pushcclosure(
            lstate,
            Some(nlua_print as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"print\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_pushinteger(lstate, 0 as lua_Integer);
    lua_setfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nlua.refcount\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    nlua_common_vim_init(lstate, thread, in_script.get());
    nlua_state_add_stdlib(lstate, true_0 != 0);
    if !in_script.get() {
        lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
        lua_pushcclosure(
            lstate,
            Some(
                nlua_thr_api_nvim__get_runtime
                    as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
            ),
            0 as ::core::ffi::c_int,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"nvim__get_runtime\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            b"api\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    lua_setfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"vim\0".as_ptr() as *const ::core::ffi::c_char,
    );
    nlua_init_packages(lstate, in_script.get());
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"package\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"loaded\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"vim\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"vim\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return lstate;
}
unsafe extern "C-unwind" fn nlua_common_free_all_mem(mut lstate: *mut lua_State) {
    let mut ref_state: *mut nlua_ref_state_t = nlua_get_ref_state(lstate);
    nlua_unref(lstate, ref_state, (*ref_state).nil_ref);
    nlua_unref(lstate, ref_state, (*ref_state).empty_dict_ref);
    lua_close(lstate);
}
unsafe extern "C" fn nlua_print_event(mut argv: *mut *mut ::core::ffi::c_void) {
    let mut msg: HlMessage = HlMessage {
        size: 0 as size_t,
        capacity: 0 as size_t,
        items: ::core::ptr::null_mut::<HlMessageChunk>(),
    };
    let mut chunk: HlMessageChunk = HlMessageChunk {
        text: String_0 {
            data: *argv.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char,
            size: ((*argv.offset(1 as ::core::ffi::c_int as isize)).expose_provenance() as intptr_t
                as size_t)
                .wrapping_sub(1 as size_t),
        },
        hl_id: 0 as ::core::ffi::c_int,
    };
    if msg.size == msg.capacity {
        msg.capacity = if msg.capacity != 0 {
            msg.capacity << 1 as ::core::ffi::c_int
        } else {
            8 as size_t
        };
        msg.items = xrealloc(
            msg.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<HlMessageChunk>().wrapping_mul(msg.capacity),
        ) as *mut HlMessageChunk;
    } else {
    };
    let c2rust_fresh0 = msg.size;
    msg.size = msg.size.wrapping_add(1);
    *msg.items.offset(c2rust_fresh0 as isize) = chunk;
    let mut needs_clear: bool = false_0 != 0;
    msg_multihl(
        object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        },
        msg,
        b"lua_print\0".as_ptr() as *const ::core::ffi::c_char,
        true_0 != 0,
        false_0 != 0,
        ::core::ptr::null_mut::<MessageData>(),
        &raw mut needs_clear,
    );
}
unsafe extern "C-unwind" fn nlua_print(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut is_thread: bool = false;
    let nargs: ::core::ffi::c_int = lua_gettop(lstate);
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"tostring\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut errmsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut errmsg_len: size_t = 0 as size_t;
    let mut msg_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    ga_init(
        &raw mut msg_ga,
        1 as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let mut curargidx: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    '_nlua_print_error: {
        while curargidx <= nargs {
            lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
            lua_pushvalue(lstate, curargidx);
            if lua_pcall(
                lstate,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
            ) != 0
            {
                errmsg = lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut errmsg_len);
                break '_nlua_print_error;
            } else {
                let mut len: size_t = 0;
                let s: *const ::core::ffi::c_char =
                    lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len);
                if s.is_null() {
                    errmsg = b"<Unknown error: lua_tolstring returned NULL for tostring result>\0"
                        .as_ptr() as *const ::core::ffi::c_char;
                    errmsg_len = ::core::mem::size_of::<[::core::ffi::c_char; 65]>()
                        .wrapping_sub(1 as usize) as size_t;
                    break '_nlua_print_error;
                } else {
                    ga_concat_len(&raw mut msg_ga, s, len);
                    if curargidx < nargs {
                        ga_append(&raw mut msg_ga, ' ' as uint8_t);
                    }
                    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    curargidx += 1;
                }
            }
        }
        ga_append(&raw mut msg_ga, NUL as uint8_t);
        lua_getfield(
            lstate,
            LUA_REGISTRYINDEX,
            b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
        );
        is_thread = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        if is_thread {
            loop_schedule_deferred(
                main_loop.ptr(),
                Event {
                    handler: Some(
                        nlua_print_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        msg_ga.ga_data,
                        ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                            msg_ga.ga_len as intptr_t as usize,
                        ),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ],
                },
            );
        } else if in_fast_callback.get() != 0 {
            multiqueue_put_event(
                (*main_loop.ptr()).events,
                Event {
                    handler: Some(
                        nlua_print_event
                            as unsafe extern "C" fn(*mut *mut ::core::ffi::c_void) -> (),
                    ),
                    argv: [
                        msg_ga.ga_data,
                        ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                            msg_ga.ga_len as intptr_t as usize,
                        ),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                        ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    ],
                },
            );
        } else {
            let mut c2rust_lvalue: [*mut ::core::ffi::c_void; 2] = [
                msg_ga.ga_data,
                ::core::ptr::with_exposed_provenance_mut::<::core::ffi::c_void>(
                    msg_ga.ga_len as intptr_t as usize,
                ),
            ];
            nlua_print_event(&raw mut c2rust_lvalue as *mut *mut ::core::ffi::c_void);
        }
        return 0 as ::core::ffi::c_int;
    }
    ga_clear(&raw mut msg_ga);
    let mut buff: *mut ::core::ffi::c_char = xmalloc(IOSIZE as size_t) as *mut ::core::ffi::c_char;
    let mut fmt: *const ::core::ffi::c_char = gettext(
        b"E5114: Converting print argument #%i: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut len_0: size_t = vim_snprintf(
        buff,
        IOSIZE as size_t,
        fmt,
        curargidx,
        errmsg_len as ::core::ffi::c_int,
        errmsg,
    ) as size_t;
    lua_pushlstring(lstate, buff, len_0);
    xfree(buff as *mut ::core::ffi::c_void);
    return lua_error(lstate);
}
unsafe extern "C-unwind" fn nlua_require(lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut name: *const ::core::ffi::c_char = luaL_checklstring(
        lstate,
        1 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<size_t>(),
    );
    lua_settop(lstate, 1 as ::core::ffi::c_int);
    lua_getfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"_LOADED\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(lstate, 2 as ::core::ffi::c_int, name);
    if lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0 {
        return 1 as ::core::ffi::c_int;
    }
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    nlua_pushref(lstate, require_ref.get());
    lua_insert(lstate, 1 as ::core::ffi::c_int);
    if (*time_fd.ptr()).is_null() {
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"require\0".as_ptr() as *const ::core::ffi::c_char,
        );
        if lua_iscfunction(lstate, -1 as ::core::ffi::c_int) != 0
            && lua_tocfunction(lstate, -1 as ::core::ffi::c_int).is_some_and(|f| {
                ::core::ptr::fn_addr_eq(
                    f,
                    nlua_require
                        as unsafe extern "C-unwind" fn(*mut lua_State) -> ::core::ffi::c_int,
                )
            })
        {
            lua_pushvalue(lstate, 1 as ::core::ffi::c_int);
            lua_setfield(
                lstate,
                LUA_GLOBALSINDEX,
                b"require\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        lua_call(lstate, 1 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        return 1 as ::core::ffi::c_int;
    }
    let mut rel_time: proftime_T = 0;
    let mut start_time: proftime_T = 0;
    (rel_time, start_time) = time_push();
    let mut status: ::core::ffi::c_int = lua_pcall(
        lstate,
        1 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    if status == 0 as ::core::ffi::c_int {
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            b"require('%s')\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        );
        time_msg(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            &raw mut start_time,
        );
    }
    time_pop(rel_time);
    return if status == 0 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        lua_error(lstate)
    };
}
unsafe extern "C-unwind" fn nlua_debug(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let input_args: [typval_T; 2] = [
        typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_FIXED,
            vval: typval_vval_union {
                v_string: b"lua_debug> \0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            },
        },
        typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        },
    ];
    loop {
        lua_settop(lstate, 0 as ::core::ffi::c_int);
        let mut input: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        get_user_input(
            &raw const input_args as *const typval_T,
            &raw mut input,
            false_0 != 0,
            false_0 != 0,
        );
        if ui_has(kUICmdline) {
            snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"lua_debug> %s\0".as_ptr() as *const ::core::ffi::c_char,
                input.vval.v_string,
            );
            ui_ext_cmdline_block_append(0 as size_t, IObuff.ptr() as *mut ::core::ffi::c_char);
        } else {
            msg_putchar('\n' as ::core::ffi::c_int);
        }
        if input.v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            || input.vval.v_string.is_null()
            || *input.vval.v_string as ::core::ffi::c_int == NUL
            || strcmp(
                input.vval.v_string,
                b"cont\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0 as ::core::ffi::c_int
        {
            tv_clear(&raw mut input);
            if ui_has(kUICmdline) {
                ui_ext_cmdline_block_leave();
            }
            return 0 as ::core::ffi::c_int;
        }
        if luaL_loadbuffer(
            lstate,
            input.vval.v_string,
            strlen(input.vval.v_string),
            b"=(debug command)\0".as_ptr() as *const ::core::ffi::c_char,
        ) != 0
        {
            nlua_error(
                lstate,
                gettext(b"E5115: Loading Lua debug string: %.*s\0".as_ptr()
                    as *const ::core::ffi::c_char),
            );
        } else if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5116: Calling Lua debug string: %.*s\0".as_ptr()
                    as *const ::core::ffi::c_char),
            );
        }
        tv_clear(&raw mut input);
    }
}
pub unsafe extern "C-unwind" fn nlua_in_fast_event(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    lua_pushboolean(
        lstate,
        (in_fast_callback.get() > 0 as ::core::ffi::c_int) as ::core::ffi::c_int,
    );
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn viml_func_is_fast(mut name: *const ::core::ffi::c_char) -> bool {
    let fdef: *const EvalFuncDef = find_internal_func(name);
    if !fdef.is_null() {
        return (*fdef).fast;
    }
    return false_0 != 0;
}
pub unsafe extern "C-unwind" fn nlua_call(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut save_current_sctx: sctx_T = sctx_T {
        sc_sid: 0,
        sc_seq: 0,
        sc_lnum: 0,
        sc_chan: 0,
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut name_len: size_t = 0;
    let mut name: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 1 as ::core::ffi::c_int, &raw mut name_len);
    if !nlua_is_deferred_safe() && !viml_func_is_fast(name) {
        let mut length: size_t = (if strlen(name) < 100 as size_t {
            strlen(name)
        } else {
            100 as size_t
        })
        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 22]>());
        vim_snprintf(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            length,
            b"Vimscript function \"%s\"\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        );
        let mut ret: ::core::ffi::c_int = luaL_error(
            lstate,
            &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
            IObuff.ptr() as *mut ::core::ffi::c_char,
        );
        return ret;
    }
    let mut nargs: ::core::ffi::c_int = lua_gettop(lstate) - 1 as ::core::ffi::c_int;
    if nargs > MAX_FUNC_ARGS as ::core::ffi::c_int {
        return luaL_error(
            lstate,
            b"Function called with too many arguments\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    let mut vim_args: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_free_vim_args: {
        while i < nargs {
            lua_pushvalue(lstate, i + 2 as ::core::ffi::c_int);
            if !nlua_pop_typval(
                lstate,
                (&raw mut vim_args as *mut typval_T).offset(i as isize),
            ) {
                api_set_error(
                    &raw mut err,
                    kErrorTypeException,
                    b"error converting argument %d\0".as_ptr() as *const ::core::ffi::c_char,
                    i + 1 as ::core::ffi::c_int,
                );
                break '_free_vim_args;
            } else {
                i += 1;
            }
        }
        force_abort.set(false_0 != 0);
        suppress_errthrow.set(false_0 != 0);
        did_throw.set(false_0 != 0);
        did_emsg.set(false_0);
        rettv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true_0 != 0;
        save_current_sctx = api_set_sctx(LUA_INTERNAL_CALL);
        let mut tstate: TryState = TryState {
            current_exception: ::core::ptr::null_mut::<except_T>(),
            private_msg_list: ::core::ptr::null_mut::<msglist_T>(),
            msg_list: ::core::ptr::null::<*const msglist_T>(),
            got_int: 0,
            did_throw: false,
            need_rethrow: 0,
            did_emsg: 0,
        };
        try_enter(&raw mut tstate);
        call_func(
            name,
            name_len as ::core::ffi::c_int,
            &raw mut rettv,
            nargs,
            &raw mut vim_args as *mut typval_T,
            &raw mut funcexe,
        );
        try_leave(&raw mut tstate, &raw mut err);
        current_sctx.set(save_current_sctx);
        if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
            nlua_push_typval(lstate, &raw mut rettv, 0 as ::core::ffi::c_int);
        }
        tv_clear(&raw mut rettv);
    }
    while i > 0 as ::core::ffi::c_int {
        i -= 1;
        tv_clear((&raw mut vim_args as *mut typval_T).offset(i as isize));
    }
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        return lua_error(lstate);
    }
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_rpcrequest(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    if !nlua_is_deferred_safe() {
        return luaL_error(
            lstate,
            &raw const e_fast_api_disabled as *const ::core::ffi::c_char,
            b"rpcrequest\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    return nlua_rpc(lstate, true_0 != 0);
}
unsafe extern "C-unwind" fn nlua_rpcnotify(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    return nlua_rpc(lstate, false_0 != 0);
}
unsafe extern "C-unwind" fn nlua_rpc(
    mut lstate: *mut lua_State,
    mut request: bool,
) -> ::core::ffi::c_int {
    let mut name_len: size_t = 0;
    let mut chan_id: uint64_t = luaL_checkinteger(lstate, 1 as ::core::ffi::c_int) as uint64_t;
    let mut name: *const ::core::ffi::c_char =
        luaL_checklstring(lstate, 2 as ::core::ffi::c_int, &raw mut name_len);
    let mut nargs: ::core::ffi::c_int = lua_gettop(lstate) - 2 as ::core::ffi::c_int;
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut args: Array = arena_array(&raw mut arena, nargs as size_t);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    '_check_err: {
        while i < nargs {
            lua_pushvalue(lstate, i + 3 as ::core::ffi::c_int);
            if args.size == args.capacity {
                args.capacity = if args.capacity != 0 {
                    args.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                args.items = xrealloc(
                    args.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(args.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh1 = args.size;
            args.size = args.size.wrapping_add(1);
            *args.items.offset(c2rust_fresh1 as isize) =
                nlua_pop_Object(lstate, false, &raw mut arena, &raw mut err);
            if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                break '_check_err;
            }
            i += 1;
        }
        if request {
            let mut res_mem: ArenaMem = ::core::ptr::null_mut::<consumed_blk>();
            let mut result: Object =
                rpc_send_call(chan_id, name, args, &raw mut res_mem, &raw mut err);
            if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int) {
                nlua_push_Object(lstate, &raw mut result, 0 as ::core::ffi::c_int);
                arena_mem_free(res_mem);
            }
        } else if !rpc_send_event(chan_id, name, args) {
            api_set_error(
                &raw mut err,
                kErrorTypeValidation,
                b"Invalid channel: %lu\0".as_ptr() as *const ::core::ffi::c_char,
                chan_id,
            );
        }
    }
    arena_mem_free(arena_finish(&raw mut arena));
    if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        lua_pushstring(lstate, err.msg);
        api_clear_error(&raw mut err);
        return lua_error(lstate);
    }
    return if request as ::core::ffi::c_int != 0 {
        1 as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    };
}
unsafe extern "C-unwind" fn nlua_nil_tostring(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    lua_pushstring(lstate, b"vim.NIL\0".as_ptr() as *const ::core::ffi::c_char);
    return 1 as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_empty_dict_tostring(
    mut lstate: *mut lua_State,
) -> ::core::ffi::c_int {
    lua_pushstring(
        lstate,
        b"vim.empty_dict()\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_ref(
    mut lstate: *mut lua_State,
    mut ref_state: *mut nlua_ref_state_t,
    mut index: ::core::ffi::c_int,
) -> LuaRef {
    lua_pushvalue(lstate, index);
    let mut ref_0: LuaRef = luaL_ref(lstate, LUA_REGISTRYINDEX);
    if ref_0 > 0 as ::core::ffi::c_int {
        (*ref_state).ref_count += 1;
    }
    return ref_0;
}
pub unsafe extern "C-unwind" fn nlua_ref_global(
    mut lstate: *mut lua_State,
    mut index: ::core::ffi::c_int,
) -> LuaRef {
    return nlua_ref(lstate, nlua_global_refs.get(), index);
}
pub unsafe extern "C-unwind" fn nlua_unref(
    mut lstate: *mut lua_State,
    mut ref_state: *mut nlua_ref_state_t,
    mut ref_0: LuaRef,
) {
    if ref_0 > 0 as ::core::ffi::c_int {
        (*ref_state).ref_count -= 1;
        luaL_unref(lstate, LUA_REGISTRYINDEX, ref_0 as ::core::ffi::c_int);
    }
}
pub unsafe extern "C-unwind" fn nlua_unref_global(mut lstate: *mut lua_State, mut ref_0: LuaRef) {
    nlua_unref(lstate, nlua_global_refs.get(), ref_0);
}
pub unsafe extern "C-unwind" fn api_free_luaref(mut ref_0: LuaRef) {
    nlua_unref_global(global_lstate.get(), ref_0);
}
pub unsafe extern "C-unwind" fn nlua_pushref(mut lstate: *mut lua_State, mut ref_0: LuaRef) {
    lua_rawgeti(lstate, LUA_REGISTRYINDEX, ref_0 as ::core::ffi::c_int);
}
pub unsafe extern "C-unwind" fn api_new_luaref(mut original_ref: LuaRef) -> LuaRef {
    if original_ref == LUA_NOREF {
        return LUA_NOREF;
    }
    let lstate: *mut lua_State = global_lstate.get();
    nlua_pushref(lstate, original_ref);
    let mut new_ref: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return new_ref;
}
pub unsafe extern "C-unwind" fn nlua_typval_eval(
    str: String_0,
    arg: *mut typval_T,
    ret_tv: *mut typval_T,
) {
    let lcmd_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 32]>()
        .wrapping_sub(1 as size_t)
        .wrapping_add(str.size)
        .wrapping_add(1 as size_t);
    let mut lcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lcmd_len < IOSIZE as size_t {
        lcmd = IObuff.ptr() as *mut ::core::ffi::c_char;
    } else {
        lcmd = xmalloc(lcmd_len) as *mut ::core::ffi::c_char;
    }
    memcpy(
        lcmd as *mut ::core::ffi::c_void,
        EVALHEADER.as_ptr() as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as size_t),
    );
    memcpy(
        lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 32]>() as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        str.data as *const ::core::ffi::c_void,
        str.size,
    );
    *lcmd.offset(lcmd_len.wrapping_sub(1 as size_t) as isize) = ')' as ::core::ffi::c_char;
    nlua_typval_exec(
        lcmd,
        lcmd_len,
        b"luaeval()\0".as_ptr() as *const ::core::ffi::c_char,
        arg,
        1 as ::core::ffi::c_int,
        true_0 != 0,
        ret_tv,
    );
    if lcmd != IObuff.ptr() as *mut ::core::ffi::c_char {
        xfree(lcmd as *mut ::core::ffi::c_void);
    }
}
pub const EVALHEADER: [::core::ffi::c_char; 32] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
        *b"local _A=select(1,...) return (\0",
    )
};
pub unsafe extern "C-unwind" fn nlua_typval_call(
    mut str: *const ::core::ffi::c_char,
    mut len: size_t,
    args: *mut typval_T,
    mut argcount: ::core::ffi::c_int,
    mut ret_tv: *mut typval_T,
) {
    let lcmd_len: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 8]>()
        .wrapping_sub(1 as size_t)
        .wrapping_add(len)
        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 6]>())
        .wrapping_sub(1 as size_t);
    let mut lcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lcmd_len < IOSIZE as size_t {
        lcmd = IObuff.ptr() as *mut ::core::ffi::c_char;
    } else {
        lcmd = xmalloc(lcmd_len) as *mut ::core::ffi::c_char;
    }
    memcpy(
        lcmd as *mut ::core::ffi::c_void,
        CALLHEADER.as_ptr() as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
    );
    memcpy(
        lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 8]>() as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        len,
    );
    memcpy(
        lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 8]>() as isize)
            .offset(-(1 as ::core::ffi::c_int as isize))
            .offset(len as isize) as *mut ::core::ffi::c_void,
        CALLSUFFIX.as_ptr() as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
    );
    nlua_typval_exec(
        lcmd,
        lcmd_len,
        b"v:lua\0".as_ptr() as *const ::core::ffi::c_char,
        args,
        argcount,
        false_0 != 0,
        ret_tv,
    );
    if lcmd != IObuff.ptr() as *mut ::core::ffi::c_char {
        xfree(lcmd as *mut ::core::ffi::c_void);
    }
}
pub const CALLHEADER: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"return \0") };
pub const CALLSUFFIX: [::core::ffi::c_char; 6] =
    unsafe { ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"(...)\0") };
pub unsafe extern "C-unwind" fn nlua_call_user_expand_func(
    mut xp: *mut expand_T,
    mut ret_tv: *mut typval_T,
) {
    let lstate: *mut lua_State = global_lstate.get();
    nlua_pushref(lstate, (*xp).xp_luaref);
    lua_pushstring(lstate, (*xp).xp_pattern);
    lua_pushstring(lstate, (*xp).xp_line);
    lua_pushinteger(lstate, (*xp).xp_col as lua_Integer);
    if nlua_pcall(lstate, 3 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
        nlua_error(
            lstate,
            gettext(b"E5108: Lua function: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return;
    }
    nlua_pop_typval(lstate, ret_tv);
}
unsafe extern "C-unwind" fn nlua_typval_exec(
    mut lcmd: *const ::core::ffi::c_char,
    mut lcmd_len: size_t,
    mut name: *const ::core::ffi::c_char,
    args: *mut typval_T,
    mut argcount: ::core::ffi::c_int,
    mut special: bool,
    mut ret_tv: *mut typval_T,
) {
    if check_secure() {
        if !ret_tv.is_null() {
            (*ret_tv).v_type = VAR_NUMBER;
            (*ret_tv).vval.v_number = 0 as varnumber_T;
        }
        return;
    }
    let lstate: *mut lua_State = global_lstate.get();
    if luaL_loadbuffer(lstate, lcmd, lcmd_len, name) != 0 {
        nlua_error(
            lstate,
            gettext(b"E5107: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < argcount {
        if (*args.offset(i as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lua_pushnil(lstate);
        } else {
            nlua_push_typval(
                lstate,
                args.offset(i as isize),
                if special as ::core::ffi::c_int != 0 {
                    kNluaPushSpecial as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        }
        i += 1;
    }
    if nlua_pcall(
        lstate,
        argcount,
        if !ret_tv.is_null() {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    ) != 0
    {
        nlua_error(
            lstate,
            gettext(b"E5108: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return;
    }
    if !ret_tv.is_null() {
        nlua_pop_typval(lstate, ret_tv);
    }
}
pub unsafe extern "C-unwind" fn nlua_exec_ga(
    mut ga: *mut garray_T,
    mut name: *mut ::core::ffi::c_char,
) {
    let mut code: *mut ::core::ffi::c_char =
        ga_concat_strings(ga, b"\n\0".as_ptr() as *const ::core::ffi::c_char);
    let mut len: size_t = strlen(code);
    nlua_typval_exec(
        code,
        len,
        name,
        ::core::ptr::null_mut::<typval_T>(),
        0 as ::core::ffi::c_int,
        false_0 != 0,
        ::core::ptr::null_mut::<typval_T>(),
    );
    xfree(code as *mut ::core::ffi::c_void);
}
pub unsafe extern "C-unwind" fn typval_exec_lua_callable(
    mut lua_cb: LuaRef,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut lstate: *mut lua_State = global_lstate.get();
    nlua_pushref(lstate, lua_cb);
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < argcount {
        if (*argvars.offset(i as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            lua_pushnil(lstate);
        } else {
            nlua_push_typval(
                lstate,
                argvars.offset(i as isize),
                if false {
                    kNluaPushSpecial as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        }
        i += 1;
    }
    if nlua_pcall(lstate, argcount, 1 as ::core::ffi::c_int) != 0 {
        nlua_error(
            lstate,
            gettext(b"Lua callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return FCERR_OTHER as ::core::ffi::c_int;
    }
    nlua_pop_typval(lstate, rettv);
    return FCERR_NONE as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_exec(
    str: String_0,
    mut chunkname: *const ::core::ffi::c_char,
    args: Array,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let lstate: *mut lua_State = global_lstate.get();
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    let mut name: *const ::core::ffi::c_char = if !chunkname.is_null()
        && *chunkname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
    {
        chunkname
    } else {
        b"<nvim>\0".as_ptr() as *const ::core::ffi::c_char
    };
    if luaL_loadbuffer(lstate, str.data, str.size, name) != 0 {
        let mut len: size_t = 0;
        let mut errstr: *const ::core::ffi::c_char =
            lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len);
        api_set_error(
            err,
            kErrorTypeValidation,
            b"Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
            len as ::core::ffi::c_int,
            errstr,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        };
    }
    let mut i: size_t = 0 as size_t;
    while i < args.size {
        nlua_push_Object(
            lstate,
            args.items.offset(i as isize),
            0 as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
    if nlua_pcall(
        lstate,
        args.size as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    ) != 0
    {
        let mut len_0: size_t = 0;
        let mut errstr_0: *const ::core::ffi::c_char =
            lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len_0);
        api_set_error(
            err,
            kErrorTypeException,
            b"Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
            len_0 as ::core::ffi::c_int,
            errstr_0,
        );
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        };
    }
    return nlua_call_pop_retval(lstate, mode, arena, top, err);
}
pub unsafe extern "C-unwind" fn nlua_ref_is_function(mut ref_0: LuaRef) -> bool {
    let lstate: *mut lua_State = global_lstate.get();
    nlua_pushref(lstate, ref_0);
    let mut is_function: bool = lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION;
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    return is_function;
}
pub unsafe extern "C-unwind" fn nlua_call_ref(
    mut ref_0: LuaRef,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    return nlua_call_ref_ctx(false_0 != 0, ref_0, name, args, mode, arena, err);
}
unsafe extern "C-unwind" fn mode_ret(mut mode: LuaRetMode) -> ::core::ffi::c_int {
    return if mode as ::core::ffi::c_uint == kRetMulti as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        LUA_MULTRET
    } else {
        1 as ::core::ffi::c_int
    };
}
pub unsafe extern "C-unwind" fn nlua_call_ref_ctx(
    mut fast: bool,
    mut ref_0: LuaRef,
    mut name: *const ::core::ffi::c_char,
    mut args: Array,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut err: *mut Error,
) -> Object {
    let lstate: *mut lua_State = global_lstate.get();
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    nlua_pushref(lstate, ref_0);
    let mut nargs: ::core::ffi::c_int = args.size as ::core::ffi::c_int;
    if !name.is_null() {
        lua_pushstring(lstate, name);
        nargs += 1;
    }
    let mut i: size_t = 0 as size_t;
    while i < args.size {
        nlua_push_Object(
            lstate,
            args.items.offset(i as isize),
            0 as ::core::ffi::c_int,
        );
        i = i.wrapping_add(1);
    }
    if fast {
        if nlua_fast_cfpcall(lstate, nargs, mode_ret(mode), -1 as ::core::ffi::c_int)
            < 0 as ::core::ffi::c_int
        {
            api_set_error(
                err,
                kErrorTypeException,
                b"fast context failure\0".as_ptr() as *const ::core::ffi::c_char,
            );
            return object {
                type_0: kObjectTypeNil,
                data: C2Rust_Unnamed_11 { boolean: false },
            };
        }
    } else if nlua_pcall(lstate, nargs, mode_ret(mode)) != 0 {
        if !err.is_null() {
            let mut len: size_t = 0;
            let mut errstr: *const ::core::ffi::c_char =
                lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut len);
            api_set_error(
                err,
                kErrorTypeException,
                b"Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char,
                len as ::core::ffi::c_int,
                errstr,
            );
        } else {
            nlua_error(
                lstate,
                gettext(b"Lua callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
        }
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        };
    }
    return nlua_call_pop_retval(lstate, mode, arena, top, err);
}
unsafe extern "C-unwind" fn nlua_call_pop_retval(
    mut lstate: *mut lua_State,
    mut mode: LuaRetMode,
    mut arena: *mut Arena,
    mut pretop: ::core::ffi::c_int,
    mut err: *mut Error,
) -> Object {
    if mode as ::core::ffi::c_uint != kRetMulti as ::core::ffi::c_int as ::core::ffi::c_uint
        && lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TNIL
    {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed_11 { boolean: false },
        };
    }
    let mut dummy: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut perr: *mut Error = if !err.is_null() { err } else { &raw mut dummy };
    match mode as ::core::ffi::c_uint {
        1 => {
            let mut bool_value: bool = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return object {
                type_0: kObjectTypeBoolean,
                data: C2Rust_Unnamed_11 {
                    boolean: bool_value,
                },
            };
        }
        2 => {
            let mut ref_0: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            return object {
                type_0: kObjectTypeLuaRef,
                data: C2Rust_Unnamed_11 { luaref: ref_0 },
            };
        }
        0 => return nlua_pop_Object(lstate, false_0 != 0, arena, perr),
        3 => {
            let mut nres: ::core::ffi::c_int = lua_gettop(lstate) - pretop;
            let mut res: Array = arena_array(arena, nres as size_t);
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < nres {
                *res.items
                    .offset((nres - i - 1 as ::core::ffi::c_int) as isize) =
                    nlua_pop_Object(lstate, false_0 != 0, arena, perr);
                if (*perr).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    return object {
                        type_0: kObjectTypeNil,
                        data: C2Rust_Unnamed_11 { boolean: false },
                    };
                }
                i += 1;
            }
            res.size = nres as size_t;
            return object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed_11 { array: res },
            };
        }
        _ => {}
    }
    unreachable!();
}
pub unsafe extern "C-unwind" fn nlua_is_deferred_safe() -> bool {
    return in_fast_callback.get() == 0 as ::core::ffi::c_int;
}
pub unsafe fn ex_lua(eap: *mut exarg_T) {
    if *(*eap).arg as ::core::ffi::c_int == NUL {
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            cmd_source_buffer(eap, true_0 != 0);
        } else {
            emsg(gettext(&raw const e_argreq as *const ::core::ffi::c_char));
        }
        return;
    }
    let mut len: size_t = 0;
    let mut code: *mut ::core::ffi::c_char = script_get(eap, &raw mut len);
    if (*eap).skip != 0 || code.is_null() {
        xfree(code as *mut ::core::ffi::c_void);
        return;
    }
    if (*eap).cmdidx as ::core::ffi::c_int == CMD_equal as ::core::ffi::c_int
        || *code.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '=' as ::core::ffi::c_int
    {
        let mut off: size_t =
            (if (*eap).cmdidx as ::core::ffi::c_int == CMD_equal as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as size_t;
        len = (len as ::core::ffi::c_ulong).wrapping_add(
            ::core::mem::size_of::<[::core::ffi::c_char; 19]>()
                .wrapping_sub(1 as usize)
                .wrapping_sub(off as usize) as ::core::ffi::c_ulong,
        ) as size_t;
        let mut code_buf: *mut ::core::ffi::c_char = xmallocz(len) as *mut ::core::ffi::c_char;
        vim_snprintf(
            code_buf,
            len.wrapping_add(1 as size_t),
            b"vim._print(true, %s)\0".as_ptr() as *const ::core::ffi::c_char,
            code.offset(off as isize),
        );
        xfree(code as *mut ::core::ffi::c_void);
        code = code_buf;
    }
    nlua_typval_exec(
        code,
        len,
        b":lua\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::ptr::null_mut::<typval_T>(),
        0 as ::core::ffi::c_int,
        false_0 != 0,
        ::core::ptr::null_mut::<typval_T>(),
    );
    xfree(code as *mut ::core::ffi::c_void);
}
pub unsafe fn ex_luado(eap: *mut exarg_T) {
    if u_save((*eap).line1 - 1 as linenr_T, (*eap).line2 + 1 as linenr_T) == FAIL {
        emsg(gettext(
            b"cannot save undo information\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let cmd: *const ::core::ffi::c_char = (*eap).arg;
    let cmd_len: size_t = strlen(cmd);
    let lstate: *mut lua_State = global_lstate.get();
    let lcmd_len: size_t = cmd_len
        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 31]>().wrapping_sub(1 as size_t))
        .wrapping_add(::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t));
    let mut lcmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if lcmd_len < IOSIZE as size_t {
        lcmd = IObuff.ptr() as *mut ::core::ffi::c_char;
    } else {
        lcmd = xmalloc(lcmd_len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    }
    memcpy(
        lcmd as *mut ::core::ffi::c_void,
        DOSTART.as_ptr() as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 31]>().wrapping_sub(1 as size_t),
    );
    memcpy(
        lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 31]>() as isize)
            .offset(-(1 as ::core::ffi::c_int as isize)) as *mut ::core::ffi::c_void,
        cmd as *const ::core::ffi::c_void,
        cmd_len,
    );
    memcpy(
        lcmd.offset(::core::mem::size_of::<[::core::ffi::c_char; 31]>() as isize)
            .offset(-(1 as ::core::ffi::c_int as isize))
            .offset(cmd_len as isize) as *mut ::core::ffi::c_void,
        DOEND.as_ptr() as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
    );
    if luaL_loadbuffer(
        lstate,
        lcmd,
        lcmd_len,
        b":luado\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        nlua_error(
            lstate,
            gettext(b"E5109: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        if lcmd_len >= IOSIZE as size_t {
            xfree(lcmd as *mut ::core::ffi::c_void);
        }
        return;
    }
    if lcmd_len >= IOSIZE as size_t {
        xfree(lcmd as *mut ::core::ffi::c_void);
    }
    if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
        nlua_error(
            lstate,
            gettext(b"E5110: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return;
    }
    let was_curbuf: *mut buf_T = curbuf.get();
    let mut l: linenr_T = (*eap).line1;
    while l <= (*eap).line2 {
        if l > (*curbuf.get()).b_ml.ml_line_count {
            break;
        }
        lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
        let old_line: *const ::core::ffi::c_char = ml_get_buf(curbuf.get(), l);
        let old_line_len: colnr_T = ml_get_buf_len(curbuf.get(), l);
        lua_pushstring(lstate, old_line);
        lua_pushnumber(lstate, l as lua_Number);
        if nlua_pcall(lstate, 2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int) != 0 {
            nlua_error(
                lstate,
                gettext(b"E5111: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
            );
            break;
        } else {
            if curbuf.get() != was_curbuf || l > (*curbuf.get()).b_ml.ml_line_count {
                break;
            }
            if lua_isstring(lstate, -1 as ::core::ffi::c_int) != 0 {
                let mut new_line_len: size_t = 0;
                let new_line: *const ::core::ffi::c_char =
                    lua_tolstring(lstate, -1 as ::core::ffi::c_int, &raw mut new_line_len);
                let new_line_transformed: *mut ::core::ffi::c_char =
                    xmemdupz(new_line as *const ::core::ffi::c_void, new_line_len)
                        as *mut ::core::ffi::c_char;
                let mut i: size_t = 0 as size_t;
                while i < new_line_len {
                    if *new_line_transformed.offset(i as isize) as ::core::ffi::c_int == NUL {
                        *new_line_transformed.offset(i as isize) = '\n' as ::core::ffi::c_char;
                    }
                    i = i.wrapping_add(1);
                }
                ml_replace(l, new_line_transformed, false_0 != 0);
                inserted_bytes(
                    l,
                    0 as colnr_T,
                    old_line_len as ::core::ffi::c_int,
                    new_line_len as ::core::ffi::c_int,
                );
            }
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
            l += 1;
        }
    }
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    check_cursor(curwin.get());
    redraw_curbuf_later(UPD_NOT_VALID);
}
pub const DOSTART: [::core::ffi::c_char; 31] = unsafe {
    ::core::mem::transmute::<[u8; 31], [::core::ffi::c_char; 31]>(
        *b"return function(line, linenr) \0",
    )
};
pub const DOEND: [::core::ffi::c_char; 5] =
    unsafe { ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b" end\0") };
pub unsafe fn ex_luafile(eap: *mut exarg_T) {
    nlua_exec_file((*eap).arg);
}
pub unsafe extern "C-unwind" fn nlua_exec_file(mut path: *const ::core::ffi::c_char) -> bool {
    let lstate: *mut lua_State = global_lstate.get();
    if !strequal(path, b"-\0".as_ptr() as *const ::core::ffi::c_char) {
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"loadfile\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushstring(lstate, path);
    } else {
        let mut stdin_dup: FileDescriptor = FileDescriptor {
            fd: 0,
            buffer: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            read_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            write_pos: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            wr: false,
            eof: false,
            non_blocking: false,
            bytes_read: 0,
        };
        let mut error: ::core::ffi::c_int = file_open_stdin(&raw mut stdin_dup);
        if error != 0 {
            return false_0 != 0;
        }
        let mut sb: StringBuilder = StringBuilder {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        sb.capacity = 64 as size_t;
        sb.items = xrealloc(
            sb.items as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(sb.capacity),
        ) as *mut ::core::ffi::c_char;
        loop {
            if got_int.get() {
                file_close(&raw mut stdin_dup, false_0 != 0);
                xfree(sb.items as *mut ::core::ffi::c_void);
                sb.capacity = 0 as size_t;
                sb.size = sb.capacity;
                sb.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                return false_0 != 0;
            }
            let mut read_size: ptrdiff_t = file_read(
                &raw mut stdin_dup,
                IObuff.ptr() as *mut ::core::ffi::c_char,
                64 as size_t,
            );
            if read_size < 0 as ptrdiff_t {
                file_close(&raw mut stdin_dup, false_0 != 0);
                xfree(sb.items as *mut ::core::ffi::c_void);
                sb.capacity = 0 as size_t;
                sb.size = sb.capacity;
                sb.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
                return false_0 != 0;
            }
            if read_size > 0 as ptrdiff_t {
                if read_size as size_t > 0 as size_t {
                    if sb.capacity < sb.size.wrapping_add(read_size as size_t) {
                        sb.capacity = sb.size.wrapping_add(read_size as size_t);
                        sb.capacity = sb.capacity.wrapping_sub(1);
                        sb.capacity |= sb.capacity >> 1 as ::core::ffi::c_int;
                        sb.capacity |= sb.capacity >> 2 as ::core::ffi::c_int;
                        sb.capacity |= sb.capacity >> 4 as ::core::ffi::c_int;
                        sb.capacity |= sb.capacity >> 8 as ::core::ffi::c_int;
                        sb.capacity |= sb.capacity >> 16 as ::core::ffi::c_int;
                        sb.capacity = sb.capacity.wrapping_add(1);
                        sb.items = xrealloc(
                            sb.items as *mut ::core::ffi::c_void,
                            ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(sb.capacity),
                        ) as *mut ::core::ffi::c_char;
                    }
                    '_c2rust_label: {
                        if !sb.items.is_null() {
                        } else {
                            __assert_fail(
                                b"(sb).items\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/lua/executor.rs\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                1910 as ::core::ffi::c_uint,
                                b"_Bool nlua_exec_file(const char *)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    memcpy(
                        sb.items.offset(sb.size as isize) as *mut ::core::ffi::c_void,
                        IObuff.ptr() as *mut ::core::ffi::c_char as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul(read_size as size_t),
                    );
                    sb.size = sb.size.wrapping_add(read_size as size_t);
                }
            }
            if read_size < 64 as ptrdiff_t {
                break;
            }
        }
        if sb.size == sb.capacity {
            sb.capacity = if sb.capacity != 0 {
                sb.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            sb.items = xrealloc(
                sb.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul(sb.capacity),
            ) as *mut ::core::ffi::c_char;
        } else {
        };
        let c2rust_fresh2 = sb.size;
        sb.size = sb.size.wrapping_add(1);
        *sb.items.offset(c2rust_fresh2 as isize) = '\0' as ::core::ffi::c_char;
        file_close(&raw mut stdin_dup, false_0 != 0);
        lua_getfield(
            lstate,
            LUA_GLOBALSINDEX,
            b"loadstring\0".as_ptr() as *const ::core::ffi::c_char,
        );
        lua_pushstring(lstate, sb.items);
        xfree(sb.items as *mut ::core::ffi::c_void);
        sb.capacity = 0 as size_t;
        sb.size = sb.capacity;
        sb.items = ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if nlua_pcall(lstate, 1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int) != 0 {
        nlua_error(
            lstate,
            gettext(b"E5111: Lua: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return false_0 != 0;
    }
    if lua_type(lstate, -2 as ::core::ffi::c_int) == LUA_TNIL {
        nlua_error(
            lstate,
            gettext(b"E5112: Lua chunk: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        '_c2rust_label_0: {
            if lua_type(lstate, -1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"lua_isnil(lstate, -1)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1936 as ::core::ffi::c_uint,
                    b"_Bool nlua_exec_file(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        return false_0 != 0;
    }
    '_c2rust_label_1: {
        if lua_type(lstate, -1 as ::core::ffi::c_int) == 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"lua_isnil(lstate, -1)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1942 as ::core::ffi::c_uint,
                b"_Bool nlua_exec_file(const char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    if nlua_pcall(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        nlua_error(
            lstate,
            gettext(b"E5113: Lua chunk: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
static expand_result_array: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
pub unsafe extern "C-unwind" fn nlua_expand_pat(mut xp: *mut expand_T) {
    let mut completions: Array = Array {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<Object>(),
    };
    let lstate: *mut lua_State = global_lstate.get();
    let mut status: ::core::ffi::c_int = FAIL;
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"vim\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"_expand_pat\0".as_ptr() as *const ::core::ffi::c_char,
    );
    luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TFUNCTION);
    let mut pat: *const ::core::ffi::c_char = (*xp).xp_pattern;
    '_c2rust_label: {
        if (*xp).xp_line.offset((*xp).xp_col as isize) >= pat as *mut ::core::ffi::c_char {
        } else {
            __assert_fail(
                b"xp->xp_line + xp->xp_col >= pat\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                1971 as ::core::ffi::c_uint,
                b"void nlua_expand_pat(expand_T *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    let mut patlen: ptrdiff_t = (*xp).xp_line.offset((*xp).xp_col as isize).offset_from(pat);
    lua_pushlstring(lstate, pat, patlen as size_t);
    if nlua_pcall(lstate, 1 as ::core::ffi::c_int, 2 as ::core::ffi::c_int)
        != 0 as ::core::ffi::c_int
    {
        nlua_error(
            lstate,
            gettext(b"vim._expand_pat: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return;
    }
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut arena: Arena = ARENA_EMPTY;
    let mut prefix_len: ptrdiff_t =
        nlua_pop_Integer(lstate, &raw mut arena, &raw mut err) as ptrdiff_t;
    if !(err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int
        || prefix_len > patlen)
    {
        completions = nlua_pop_Array(lstate, &raw mut arena, &raw mut err);
        '_cleanup_array: {
            if err.type_0 as ::core::ffi::c_int == kErrorTypeNone as ::core::ffi::c_int {
                ga_clear(expand_result_array.ptr());
                ga_init(
                    expand_result_array.ptr(),
                    ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                    80 as ::core::ffi::c_int,
                );
                let mut i: size_t = 0 as size_t;
                while i < completions.size {
                    let mut v: Object = *completions.items.offset(i as isize);
                    if v.type_0 as ::core::ffi::c_uint
                        != kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        break '_cleanup_array;
                    }
                    ga_grow(expand_result_array.ptr(), 1 as ::core::ffi::c_int);
                    *((*expand_result_array.ptr()).ga_data as *mut *mut ::core::ffi::c_char)
                        .offset((*expand_result_array.ptr()).ga_len as isize) =
                        string_to_cstr(v.data.string);
                    (*expand_result_array.ptr()).ga_len += 1;
                    i = i.wrapping_add(1);
                }
                (*xp).xp_pattern = (*xp).xp_pattern.offset(prefix_len as isize);
                status = OK;
            }
        }
        arena_mem_free(arena_finish(&raw mut arena));
    }
    if status == FAIL {
        ga_clear(expand_result_array.ptr());
    }
}
pub unsafe extern "C-unwind" fn nlua_expand_get_matches(
    mut num_results: *mut ::core::ffi::c_int,
    mut results: *mut *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    *results = (*expand_result_array.ptr()).ga_data as *mut *mut ::core::ffi::c_char;
    *num_results = (*expand_result_array.ptr()).ga_len;
    expand_result_array.set(GA_EMPTY_INIT_VALUE);
    return (*num_results > 0 as ::core::ffi::c_int) as ::core::ffi::c_int;
}
unsafe extern "C-unwind" fn nlua_is_thread(mut lstate: *mut lua_State) -> ::core::ffi::c_int {
    lua_getfield(
        lstate,
        LUA_REGISTRYINDEX,
        b"nvim.thread\0".as_ptr() as *const ::core::ffi::c_char,
    );
    return 1 as ::core::ffi::c_int;
}
pub unsafe extern "C-unwind" fn nlua_is_table_from_lua(arg: *const typval_T) -> bool {
    if (*arg).v_type as ::core::ffi::c_uint == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*(*arg).vval.v_dict).lua_table_ref != LUA_NOREF;
    } else if (*arg).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return (*(*arg).vval.v_list).lua_table_ref != LUA_NOREF;
    } else {
        return false_0 != 0;
    };
}
pub unsafe extern "C-unwind" fn nlua_register_table_as_callable(
    arg: *const typval_T,
) -> *mut ::core::ffi::c_char {
    let mut table_ref: LuaRef = LUA_NOREF;
    if (*arg).v_type as ::core::ffi::c_uint == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        table_ref = (*(*arg).vval.v_dict).lua_table_ref;
    } else if (*arg).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        table_ref = (*(*arg).vval.v_list).lua_table_ref;
    }
    if table_ref == LUA_NOREF {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let lstate: *mut lua_State = global_lstate.get();
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    nlua_pushref(lstate, table_ref);
    if lua_getmetatable(lstate, -1 as ::core::ffi::c_int) == 0 {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        '_c2rust_label: {
            if top == lua_gettop(lstate) {
            } else {
                __assert_fail(
                    b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2064 as ::core::ffi::c_uint,
                    b"char *nlua_register_table_as_callable(const typval_T *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"__call\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if !(lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION) {
        lua_settop(lstate, -3 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        '_c2rust_label_0: {
            if top == lua_gettop(lstate) {
            } else {
                __assert_fail(
                    b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2071 as ::core::ffi::c_uint,
                    b"char *nlua_register_table_as_callable(const typval_T *const)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    lua_settop(lstate, -2 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    let mut func: LuaRef = nlua_ref_global(lstate, -1 as ::core::ffi::c_int);
    let mut name: *mut ::core::ffi::c_char = register_luafunc(func);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    '_c2rust_label_1: {
        if top == lua_gettop(lstate) {
        } else {
            __assert_fail(
                b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2081 as ::core::ffi::c_uint,
                b"char *nlua_register_table_as_callable(const typval_T *const)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return name;
}
pub unsafe extern "C-unwind" fn nlua_execute_on_key(
    mut c: ::core::ffi::c_int,
    mut typed_buf: *mut ::core::ffi::c_char,
) -> bool {
    static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
    if recursive.get() {
        return false_0 != 0;
    }
    recursive.set(true_0 != 0);
    let mut buf: [::core::ffi::c_char; 67] = [0; 67];
    let mut buf_len: size_t = special_to_buf(
        c,
        mod_mask.get(),
        false_0 != 0,
        &raw mut buf as *mut ::core::ffi::c_char,
    ) as size_t;
    vim_unescape_ks(typed_buf);
    let lstate: *mut lua_State = global_lstate.get();
    let mut top: ::core::ffi::c_int = lua_gettop(lstate);
    lua_getfield(
        lstate,
        LUA_GLOBALSINDEX,
        b"vim\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_getfield(
        lstate,
        -1 as ::core::ffi::c_int,
        b"_on_key\0".as_ptr() as *const ::core::ffi::c_char,
    );
    luaL_checktype(lstate, -1 as ::core::ffi::c_int, LUA_TFUNCTION);
    lua_pushlstring(lstate, &raw mut buf as *mut ::core::ffi::c_char, buf_len);
    lua_pushstring(lstate, typed_buf);
    let mut save_got_int: ::core::ffi::c_int = got_int.get() as ::core::ffi::c_int;
    got_int.set(false_0 != 0);
    let mut discard: bool = false_0 != 0;
    if lua_pcall(
        lstate,
        2 as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) != 0
    {
        nlua_error(
            lstate,
            gettext(b"vim.on_key() callbacks: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
    } else {
        if lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TBOOLEAN {
            discard = lua_toboolean(lstate, -1 as ::core::ffi::c_int) != 0;
        }
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    }
    got_int.set(got_int.get() as ::core::ffi::c_int | save_got_int != 0);
    lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
    '_c2rust_label: {
        if top == lua_gettop(lstate) {
        } else {
            __assert_fail(
                b"top == lua_gettop(lstate)\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2138 as ::core::ffi::c_uint,
                b"_Bool nlua_execute_on_key(int, char *)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    recursive.set(false_0 != 0);
    return discard;
}
pub unsafe extern "C-unwind" fn nlua_set_sctx(mut current: *mut sctx_T) {
    let mut source_path: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut sid: ::core::ffi::c_int = 0;
    if !script_is_lua((*current).sc_sid) {
        return;
    }
    (*current).sc_lnum = 0 as ::core::ffi::c_int as linenr_T;
    if p_verbose.get() <= 0 as OptInt {
        return;
    }
    let lstate: *mut lua_State = active_lstate.get();
    let mut info: *mut lua_Debug = xmalloc(::core::mem::size_of::<lua_Debug>()) as *mut lua_Debug;
    let mut level: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    '_cleanup: {
        loop {
            if lua_getstack(lstate, level, info) != 1 as ::core::ffi::c_int {
                break '_cleanup;
            }
            if lua_getinfo(
                lstate,
                b"nSl\0".as_ptr() as *const ::core::ffi::c_char,
                info,
            ) == 0 as ::core::ffi::c_int
            {
                break '_cleanup;
            }
            if !(*(*info).what.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'C' as ::core::ffi::c_int
                || *(*info).source.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '@' as ::core::ffi::c_int)
            {
                break;
            }
            level += 1;
        }
        source_path = fix_fname((*info).source.offset(1 as ::core::ffi::c_int as isize));
        sid = find_script_by_name(source_path);
        if sid > 0 as ::core::ffi::c_int {
            xfree(source_path as *mut ::core::ffi::c_void);
        } else {
            let mut si: *mut scriptitem_T = new_script_item(source_path, &raw mut sid);
            (*si).sn_lua = true_0 != 0;
        }
        (*current).sc_sid = sid as scid_T;
        (*current).sc_seq = -1 as ::core::ffi::c_int;
        (*current).sc_lnum = (*info).currentline as linenr_T;
    }
    xfree(info as *mut ::core::ffi::c_void);
}
pub unsafe extern "C-unwind" fn nlua_do_ucmd(
    mut cmd: *mut ucmd_T,
    mut eap: *mut exarg_T,
    mut preview: bool,
) -> ::core::ffi::c_int {
    let lstate: *mut lua_State = global_lstate.get();
    nlua_pushref(
        lstate,
        if preview as ::core::ffi::c_int != 0 {
            (*cmd).uc_preview_luaref
        } else {
            (*cmd).uc_luaref
        },
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushstring(lstate, (*cmd).uc_name);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"name\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        ((*eap).forceit == 1 as ::core::ffi::c_int) as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"bang\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushinteger(lstate, (*eap).line1 as lua_Integer);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"line1\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushinteger(lstate, (*eap).line2 as lua_Integer);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"line2\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushstring(lstate, (*eap).arg);
    lua_pushvalue(lstate, -1 as ::core::ffi::c_int);
    lua_setfield(
        lstate,
        -4 as ::core::ffi::c_int,
        b"args\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*cmd).uc_argt & EX_NOSPC as uint32_t != 0 {
        if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 || strlen((*eap).arg) != 0 {
            lua_rawseti(lstate, -2 as ::core::ffi::c_int, 1 as ::core::ffi::c_int);
        } else {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        }
    } else if (*eap).args.is_null() {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let mut length: size_t = strlen((*eap).arg);
        let mut end: size_t = 0 as size_t;
        let mut len: size_t = 0 as size_t;
        let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut buf: *mut ::core::ffi::c_char =
            xcalloc(length, ::core::mem::size_of::<::core::ffi::c_char>())
                as *mut ::core::ffi::c_char;
        let mut done: bool = false_0 != 0;
        while !done {
            done = uc_split_args_iter((*eap).arg, length, &raw mut end, buf, &raw mut len);
            if len > 0 as size_t {
                lua_pushlstring(lstate, buf, len);
                lua_rawseti(lstate, -2 as ::core::ffi::c_int, i);
                i += 1;
            }
        }
        xfree(buf as *mut ::core::ffi::c_void);
    } else {
        lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        let mut i_0: size_t = 0 as size_t;
        while i_0 < (*eap).argc {
            lua_pushlstring(
                lstate,
                *(*eap).args.offset(i_0 as isize),
                *(*eap).arglens.offset(i_0 as isize),
            );
            lua_rawseti(
                lstate,
                -2 as ::core::ffi::c_int,
                i_0 as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
            );
            i_0 = i_0.wrapping_add(1);
        }
    }
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"fargs\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut reg: [::core::ffi::c_char; 2] = [
        (*eap).regname as ::core::ffi::c_char,
        NUL as ::core::ffi::c_char,
    ];
    lua_pushstring(lstate, &raw mut reg as *mut ::core::ffi::c_char);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"reg\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushinteger(lstate, (*eap).addr_count as lua_Integer);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"range\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*eap).addr_count > 0 as ::core::ffi::c_int {
        lua_pushinteger(lstate, (*eap).line2 as lua_Integer);
    } else {
        lua_pushinteger(lstate, (*cmd).uc_def as lua_Integer);
    }
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"count\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut nargs: [::core::ffi::c_char; 2] = [0; 2];
    if (*cmd).uc_argt & EX_EXTRA as uint32_t != 0 {
        if (*cmd).uc_argt & EX_NOSPC as uint32_t != 0 {
            if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 {
                nargs[0 as ::core::ffi::c_int as usize] = '1' as ::core::ffi::c_char;
            } else {
                nargs[0 as ::core::ffi::c_int as usize] = '?' as ::core::ffi::c_char;
            }
        } else if (*cmd).uc_argt & EX_NEEDARG as uint32_t != 0 {
            nargs[0 as ::core::ffi::c_int as usize] = '+' as ::core::ffi::c_char;
        } else {
            nargs[0 as ::core::ffi::c_int as usize] = '*' as ::core::ffi::c_char;
        }
    } else {
        nargs[0 as ::core::ffi::c_int as usize] = '0' as ::core::ffi::c_char;
    }
    nargs[1 as ::core::ffi::c_int as usize] = NUL as ::core::ffi::c_char;
    lua_pushstring(lstate, &raw mut nargs as *mut ::core::ffi::c_char);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"nargs\0".as_ptr() as *const ::core::ffi::c_char,
    );
    let mut buf_0: [::core::ffi::c_char; 200] = [
        0 as ::core::ffi::c_char,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    uc_mods(
        &raw mut buf_0 as *mut ::core::ffi::c_char,
        cmdmod.ptr(),
        false_0 != 0,
    );
    lua_pushstring(lstate, &raw mut buf_0 as *mut ::core::ffi::c_char);
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"mods\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_createtable(lstate, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    lua_pushinteger(
        lstate,
        ((*cmdmod.ptr()).cmod_tab - 1 as ::core::ffi::c_int) as lua_Integer,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"tab\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushinteger(
        lstate,
        ((*cmdmod.ptr()).cmod_verbose - 1 as ::core::ffi::c_int) as lua_Integer,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"verbose\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if (*cmdmod.ptr()).cmod_split & WSP_ABOVE as ::core::ffi::c_int != 0 {
        lua_pushstring(
            lstate,
            b"aboveleft\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if (*cmdmod.ptr()).cmod_split & WSP_BELOW as ::core::ffi::c_int != 0 {
        lua_pushstring(
            lstate,
            b"belowright\0".as_ptr() as *const ::core::ffi::c_char,
        );
    } else if (*cmdmod.ptr()).cmod_split & WSP_TOP as ::core::ffi::c_int != 0 {
        lua_pushstring(lstate, b"topleft\0".as_ptr() as *const ::core::ffi::c_char);
    } else if (*cmdmod.ptr()).cmod_split & WSP_BOT as ::core::ffi::c_int != 0 {
        lua_pushstring(lstate, b"botright\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        lua_pushstring(lstate, b"\0".as_ptr() as *const ::core::ffi::c_char);
    }
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"split\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_split & WSP_VERT as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"vertical\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_split & WSP_HOR as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"horizontal\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_flags & CMOD_SILENT as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"silent\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_flags & CMOD_ERRSILENT as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"emsg_silent\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_flags & CMOD_UNSILENT as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"unsilent\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_flags & CMOD_SANDBOX as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"sandbox\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushboolean(
        lstate,
        (*cmdmod.ptr()).cmod_flags & CMOD_NOAUTOCMD as ::core::ffi::c_int,
    );
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"noautocmd\0".as_ptr() as *const ::core::ffi::c_char,
    );
    static mod_entries: GlobalCell<[mod_entry_T; 9]> = GlobalCell::new([
        mod_entry_T {
            flag: CMOD_BROWSE as ::core::ffi::c_int,
            name: b"browse\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_CONFIRM as ::core::ffi::c_int,
            name: b"confirm\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_HIDE as ::core::ffi::c_int,
            name: b"hide\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPALT as ::core::ffi::c_int,
            name: b"keepalt\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPJUMPS as ::core::ffi::c_int,
            name: b"keepjumps\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPMARKS as ::core::ffi::c_int,
            name: b"keepmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_KEEPPATTERNS as ::core::ffi::c_int,
            name: b"keeppatterns\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_LOCKMARKS as ::core::ffi::c_int,
            name: b"lockmarks\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        },
        mod_entry_T {
            flag: CMOD_NOSWAPFILE as ::core::ffi::c_int,
            name: b"noswapfile\0".as_ptr() as *const ::core::ffi::c_char
                as *mut ::core::ffi::c_char,
        },
    ]);
    let mut i_1: size_t = 0 as size_t;
    while i_1
        < ::core::mem::size_of::<[mod_entry_T; 9]>()
            .wrapping_div(::core::mem::size_of::<mod_entry_T>())
            .wrapping_div(
                (::core::mem::size_of::<[mod_entry_T; 9]>()
                    .wrapping_rem(::core::mem::size_of::<mod_entry_T>())
                    == 0) as ::core::ffi::c_int as usize,
            )
    {
        lua_pushboolean(
            lstate,
            (*cmdmod.ptr()).cmod_flags & (*mod_entries.ptr())[i_1 as usize].flag,
        );
        lua_setfield(
            lstate,
            -2 as ::core::ffi::c_int,
            (*mod_entries.ptr())[i_1 as usize].name,
        );
        i_1 = i_1.wrapping_add(1);
    }
    lua_setfield(
        lstate,
        -2 as ::core::ffi::c_int,
        b"smods\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if preview {
        lua_pushinteger(lstate, cmdpreview_get_ns() as lua_Integer);
        let mut cmdpreview_bufnr: handle_T = cmdpreview_get_bufnr();
        if cmdpreview_bufnr != 0 as ::core::ffi::c_int {
            lua_pushinteger(lstate, cmdpreview_bufnr as lua_Integer);
        } else {
            lua_pushnil(lstate);
        }
    }
    if nlua_pcall(
        lstate,
        if preview as ::core::ffi::c_int != 0 {
            3 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        },
        if preview as ::core::ffi::c_int != 0 {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        },
    ) != 0
    {
        nlua_error(
            lstate,
            gettext(b"Lua :command callback: %.*s\0".as_ptr() as *const ::core::ffi::c_char),
        );
        return 0 as ::core::ffi::c_int;
    }
    let mut retv: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if preview {
        if lua_isnumber(lstate, -1 as ::core::ffi::c_int) != 0
            && {
                retv = lua_tointeger(lstate, -1 as ::core::ffi::c_int) as ::core::ffi::c_int;
                retv >= 0 as ::core::ffi::c_int
            }
            && retv <= 2 as ::core::ffi::c_int
        {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        } else {
            retv = 0 as ::core::ffi::c_int;
        }
    }
    return retv;
}
pub unsafe extern "C-unwind" fn nlua_funcref_str(
    mut ref_0: LuaRef,
    mut arena: *mut Arena,
) -> *mut ::core::ffi::c_char {
    let mut ar: lua_Debug = lua_Debug {
        event: 0,
        name: ::core::ptr::null::<::core::ffi::c_char>(),
        namewhat: ::core::ptr::null::<::core::ffi::c_char>(),
        what: ::core::ptr::null::<::core::ffi::c_char>(),
        source: ::core::ptr::null::<::core::ffi::c_char>(),
        currentline: 0,
        nups: 0,
        linedefined: 0,
        lastlinedefined: 0,
        short_src: [0; 60],
        i_ci: 0,
    };
    let lstate: *mut lua_State = global_lstate.get();
    if lua_checkstack(lstate, 1 as ::core::ffi::c_int) != 0 {
        nlua_pushref(lstate, ref_0);
        if !(lua_type(lstate, -1 as ::core::ffi::c_int) == LUA_TFUNCTION) {
            lua_settop(lstate, -1 as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
        } else {
            ar = lua_Debug {
                event: 0,
                name: ::core::ptr::null::<::core::ffi::c_char>(),
                namewhat: ::core::ptr::null::<::core::ffi::c_char>(),
                what: ::core::ptr::null::<::core::ffi::c_char>(),
                source: ::core::ptr::null::<::core::ffi::c_char>(),
                currentline: 0,
                nups: 0,
                linedefined: 0,
                lastlinedefined: 0,
                short_src: [0; 60],
                i_ci: 0,
            };
            if lua_getinfo(
                lstate,
                b">S\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut ar,
            ) != 0
                && *ar.source as ::core::ffi::c_int == '@' as ::core::ffi::c_int
                && ar.linedefined >= 0 as ::core::ffi::c_int
            {
                let mut src: *mut ::core::ffi::c_char = home_replace_save(
                    ::core::ptr::null_mut::<buf_T>(),
                    ar.source.offset(1 as ::core::ffi::c_int as isize),
                );
                let mut str: String_0 = arena_printf(
                    arena,
                    b"<Lua %d: %s:%d>\0".as_ptr() as *const ::core::ffi::c_char,
                    ref_0,
                    src,
                    ar.linedefined,
                );
                xfree(src as *mut ::core::ffi::c_void);
                return str.data;
            }
        }
    }
    return arena_printf(
        arena,
        b"<Lua %d>\0".as_ptr() as *const ::core::ffi::c_char,
        ref_0,
    )
    .data;
}
pub unsafe extern "C-unwind" fn nlua_init_defaults() {
    let L: *mut lua_State = global_lstate.get();
    '_c2rust_label: {
        if !L.is_null() {
        } else {
            __assert_fail(
                b"L\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/lua/executor.rs\0".as_ptr() as *const ::core::ffi::c_char,
                2417 as ::core::ffi::c_uint,
                b"void nlua_init_defaults(void)\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    lua_getfield(
        L,
        LUA_GLOBALSINDEX,
        b"require\0".as_ptr() as *const ::core::ffi::c_char,
    );
    lua_pushstring(
        L,
        b"vim._core.defaults\0".as_ptr() as *const ::core::ffi::c_char,
    );
    if nlua_pcall(L, 1 as ::core::ffi::c_int, 0 as ::core::ffi::c_int) != 0 {
        fprintf(
            stderr,
            b"%s\n\0".as_ptr() as *const ::core::ffi::c_char,
            lua_tolstring(
                L,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<size_t>(),
            ),
        );
    }
}
pub unsafe extern "C-unwind" fn nlua_func_exists(
    mut lua_funcname: *const ::core::ffi::c_char,
) -> bool {
    let mut args: Array = ARRAY_DICT_INIT;
    let mut args__items: [Object; 1] = [Object {
        type_0: kObjectTypeNil,
        data: C2Rust_Unnamed_11 { boolean: false },
    }; 1];
    args.capacity = 1 as size_t;
    args.items = &raw mut args__items as *mut Object;
    let mut length: size_t = strlen(lua_funcname).wrapping_add(8 as size_t);
    let mut str: *mut ::core::ffi::c_char = xmalloc(length) as *mut ::core::ffi::c_char;
    vim_snprintf(
        str,
        length,
        b"return %s\0".as_ptr() as *const ::core::ffi::c_char,
        lua_funcname,
    );
    let c2rust_fresh3 = args.size;
    args.size = args.size.wrapping_add(1);
    *args.items.offset(c2rust_fresh3 as isize) = object {
        type_0: kObjectTypeString,
        data: C2Rust_Unnamed_11 {
            string: cstr_as_string(str),
        },
    };
    let mut err: Error = Error {
        type_0: kErrorTypeNone,
        msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    let mut result: Object = nlua_exec(
        String_0 {
            data: b"return type(loadstring(...)()) == 'function'\0".as_ptr()
                as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            size: ::core::mem::size_of::<[::core::ffi::c_char; 45]>().wrapping_sub(1 as size_t),
        },
        ::core::ptr::null::<::core::ffi::c_char>(),
        args,
        kRetNilBool,
        ::core::ptr::null_mut::<Arena>(),
        &raw mut err,
    );
    xfree(str as *mut ::core::ffi::c_void);
    api_clear_error(&raw mut err);
    return result.type_0 as ::core::ffi::c_uint
        == kObjectTypeBoolean as ::core::ffi::c_int as ::core::ffi::c_uint
        && result.data.boolean as ::core::ffi::c_int == true_0;
}
// LuaJIT bytecode for the builtin `vim.*` modules, compiled by build.rs
// (src/gen/compile_lua_modules.lua) from `runtime/lua/vim/`. c2rust
// originally transpiled these as ~215k lines of array literals frozen from
// upstream's generated char blobs; now the sources next to the binary are
// the sources inside it. Each blob carries gen_char_blob.lua's trailing 0
// sentinel, which is why nlua_module_preloader loads `size - 1` bytes.
const vim_dot__init_packages_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__init_packages_module.bin"
));
const vim_dot_inspect_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_inspect_module.bin"
));
const vim_dot_filetype_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_filetype_module.bin"
));
const vim_dot_fs_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_fs_module.bin"
));
const vim_dot_F_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_F_module.bin"
));
const vim_dot_keymap_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_keymap_module.bin"
));
const vim_dot_loader_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_loader_module.bin"
));
const vim_dot_text_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot_text_module.bin"
));
const vim_dot__core_dot_defaults_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_defaults_module.bin"
));
const vim_dot__core_dot_editor_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_editor_module.bin"
));
const vim_dot__core_dot_ex_cmd_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_ex_cmd_module.bin"
));
const vim_dot__core_dot_exrc_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_exrc_module.bin"
));
const vim_dot__core_dot_help_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_help_module.bin"
));
const vim_dot__core_dot_log_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_log_module.bin"
));
const vim_dot__core_dot_options_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_options_module.bin"
));
const vim_dot__core_dot_server_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_server_module.bin"
));
const vim_dot__core_dot_shared_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_shared_module.bin"
));
const vim_dot__core_dot_stringbuffer_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_stringbuffer_module.bin"
));
const vim_dot__core_dot_system_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_system_module.bin"
));
const vim_dot__core_dot_ui2_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_ui2_module.bin"
));
const vim_dot__core_dot_util_module: &[uint8_t] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/lua_modules/vim_dot__core_dot_util_module.bin"
));
static builtin_modules: SharedCell<[ModuleDef; 21]> = SharedCell::new([
    ModuleDef {
        name: b"vim._init_packages\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__init_packages_module.as_ptr(),
        size: vim_dot__init_packages_module.len(),
    },
    ModuleDef {
        name: b"vim.inspect\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_inspect_module.as_ptr(),
        size: vim_dot_inspect_module.len(),
    },
    ModuleDef {
        name: b"vim.filetype\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_filetype_module.as_ptr(),
        size: vim_dot_filetype_module.len(),
    },
    ModuleDef {
        name: b"vim.fs\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_fs_module.as_ptr(),
        size: vim_dot_fs_module.len(),
    },
    ModuleDef {
        name: b"vim.F\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_F_module.as_ptr(),
        size: vim_dot_F_module.len(),
    },
    ModuleDef {
        name: b"vim.keymap\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_keymap_module.as_ptr(),
        size: vim_dot_keymap_module.len(),
    },
    ModuleDef {
        name: b"vim.loader\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_loader_module.as_ptr(),
        size: vim_dot_loader_module.len(),
    },
    ModuleDef {
        name: b"vim.text\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot_text_module.as_ptr(),
        size: vim_dot_text_module.len(),
    },
    ModuleDef {
        name: b"vim._core.defaults\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_defaults_module.as_ptr(),
        size: vim_dot__core_dot_defaults_module.len(),
    },
    ModuleDef {
        name: b"vim._core.editor\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_editor_module.as_ptr(),
        size: vim_dot__core_dot_editor_module.len(),
    },
    ModuleDef {
        name: b"vim._core.ex_cmd\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_ex_cmd_module.as_ptr(),
        size: vim_dot__core_dot_ex_cmd_module.len(),
    },
    ModuleDef {
        name: b"vim._core.exrc\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_exrc_module.as_ptr(),
        size: vim_dot__core_dot_exrc_module.len(),
    },
    ModuleDef {
        name: b"vim._core.help\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_help_module.as_ptr(),
        size: vim_dot__core_dot_help_module.len(),
    },
    ModuleDef {
        name: b"vim._core.log\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_log_module.as_ptr(),
        size: vim_dot__core_dot_log_module.len(),
    },
    ModuleDef {
        name: b"vim._core.options\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_options_module.as_ptr(),
        size: vim_dot__core_dot_options_module.len(),
    },
    ModuleDef {
        name: b"vim._core.server\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_server_module.as_ptr(),
        size: vim_dot__core_dot_server_module.len(),
    },
    ModuleDef {
        name: b"vim._core.shared\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_shared_module.as_ptr(),
        size: vim_dot__core_dot_shared_module.len(),
    },
    ModuleDef {
        name: b"vim._core.stringbuffer\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_stringbuffer_module.as_ptr(),
        size: vim_dot__core_dot_stringbuffer_module.len(),
    },
    ModuleDef {
        name: b"vim._core.system\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_system_module.as_ptr(),
        size: vim_dot__core_dot_system_module.len(),
    },
    ModuleDef {
        name: b"vim._core.ui2\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_ui2_module.as_ptr(),
        size: vim_dot__core_dot_ui2_module.len(),
    },
    ModuleDef {
        name: b"vim._core.util\0".as_ptr() as *const ::core::ffi::c_char
            as *mut ::core::ffi::c_char,
        data: vim_dot__core_dot_util_module.as_ptr(),
        size: vim_dot__core_dot_util_module.len(),
    },
]);
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
