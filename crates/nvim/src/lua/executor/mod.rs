//! The Lua interpreter, and everything the editor reaches it through.
//!
//! The state itself is built in [`state`]; [`vim_module`] assembles the `vim`
//! table on it, [`modules`] holds the runtime Lua compiled into the binary,
//! and [`refs`] is the registry table every callback is kept in. The rest is
//! one direction of travel each: [`exec`] and [`call`] for Vimscript
//! reaching Lua, [`excmds`] and [`ucmd`] for the `:` commands, [`callable`]
//! for a Lua value Vimscript can call, [`schedule`] and [`luv`] for the event
//! loop, [`print`] for the output, [`error`] for what a failure becomes, and
//! [`expand`] for command-line completion.
//!
//! The six editor-wide cells live here because every one of those children
//! reads at least one of them.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use crate::global_cell::{GlobalCell, SharedCell};
use crate::lua::ffi::LUA_REFNIL;
use crate::types::{
    LuaRef, LuaRetMode, dict_T, funcexe_T, garray_T, lua_CFunction, lua_State, partial_T, typval_T,
    uint64_t, uv_thread_t,
};

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

// -- The foreign surface ----------------------------------------------------
//
// The debug and thread halves of the Lua and luv APIs, which `lua/ffi.rs`
// does not carry because nothing outside this module names them.

unsafe extern "C" {
    pub(crate) fn lua_getstack(L: *mut lua_State, level: c_int, ar: *mut lua_Debug) -> c_int;
    pub(crate) fn lua_getinfo(L: *mut lua_State, what: *const c_char, ar: *mut lua_Debug) -> c_int;
    pub(crate) fn luv_set_callback(L: *mut lua_State, pcall: luv_CFpcall);
    pub(crate) fn luv_set_thread(L: *mut lua_State, pcall: luv_CFpcall);
    pub(crate) fn luv_set_cthread(L: *mut lua_State, cpcall: luv_CFcpcall);
    pub(crate) fn luv_set_thread_cb(acquire: luv_acquire_vm, release: luv_release_vm);
}

/// What `lua_getinfo` fills in: `lua.h`'s `lua_Debug`, layout and all.
#[repr(C)]
pub struct lua_Debug {
    pub event: c_int,
    pub name: *const c_char,
    pub namewhat: *const c_char,
    pub what: *const c_char,
    pub source: *const c_char,
    pub currentline: c_int,
    pub nups: c_int,
    pub linedefined: c_int,
    pub lastlinedefined: c_int,
    pub short_src: [c_char; 60],
    pub i_ci: c_int,
}

/// luv's protected-call hook: `(L, nargs, nresults, flags)`.
pub type luv_CFpcall =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, c_int, c_int, c_int) -> c_int>;
/// luv's C-function protected-call hook.
pub type luv_CFcpcall =
    Option<unsafe extern "C-unwind" fn(*mut lua_State, lua_CFunction, *mut c_void, c_int) -> c_int>;
pub type luv_acquire_vm = Option<unsafe extern "C-unwind" fn() -> *mut lua_State>;
pub type luv_release_vm = Option<unsafe extern "C-unwind" fn(*mut lua_State)>;

/// Which callback an error reported through the event queue came from.
pub type luv_err_t = luv_err_type;
pub type luv_err_type = ::core::ffi::c_uint;
pub const kCallback: luv_err_type = 0;
pub const kThread: luv_err_type = 1;
pub const kThreadCallback: luv_err_type = 2;

/// luv's "do not `preserve_exit` on out of memory" flag.
pub const LUVF_CALLBACK_NOEXIT: c_int = 0x1;

/// What a Lua call's result is converted to.
pub const kRetObject: LuaRetMode = 0;
pub const kRetNilBool: LuaRetMode = 1;
pub const kRetLuaref: LuaRetMode = 2;
pub const kRetMulti: LuaRetMode = 3;

/// `call_func`'s answers: the two this module can produce.
pub type FuncErrorType = ::core::ffi::c_uint;
pub const FCERR_NONE: FuncErrorType = 5;
pub const FCERR_OTHER: FuncErrorType = 6;

/// The most arguments `vim.call()` will convert.
pub const MAX_FUNC_ARGS: ::core::ffi::c_uint = 20;

/// The channel id an api call made from inside the editor carries.
const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

/// An empty garray, growing one item at a time.
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 1,
    ga_data: ::core::ptr::null_mut(),
};

/// A zeroed `funcexe_T`, which `nlua_call` fills.
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0,
    fe_lastline: 0,
    fe_doesrange: ::core::ptr::null_mut(),
    fe_evaluate: false,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false,
};

// -- The editor-wide state --------------------------------------------------

/// How deep inside a *fast* callback we are: a libuv callback running on the
/// main loop, where nothing that touches the editor's own state may run.
/// Counted rather than flagged, because one may nest inside another.
static in_fast_callback: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether this process is `nvim -l`, with no editor at all: errors go to
/// stderr and the runtime path is never rebuilt.
static in_script: SharedCell<bool> = SharedCell::new(false);

/// The editor's own Lua state.
static global_lstate: GlobalCell<*mut lua_State> = GlobalCell::new(::core::ptr::null_mut());

/// The state a callback is currently running on, which is [`global_lstate`]
/// except inside a luv thread.
pub static active_lstate: GlobalCell<*mut lua_State> = GlobalCell::new(::core::ptr::null_mut());

/// The stock `require`, kept so `--startuptime`'s wrapper can delegate to it.
static require_ref: GlobalCell<LuaRef> = GlobalCell::new(LUA_REFNIL);

/// The thread `nlua_init` ran on: the only one allowed to rebuild the
/// runtime search path.
static main_thread: SharedCell<uv_thread_t> = SharedCell::new(0);

/// The editor's Lua state, for the api code that has no state to hand.
pub fn get_global_lstate() -> *mut lua_State {
    global_lstate.get()
}
