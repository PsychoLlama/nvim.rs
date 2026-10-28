//! The Lua interpreter, and everything the editor reaches it through.
//!
//! The state itself is built in [`state`]; [`vim_module`] assembles the `vim`
//! table on it, [`modules`] holds the runtime Lua compiled into the binary,
//! and [`refs`] is the registry table every callback is kept in. The rest is
//! one direction of travel each: [`exec`] and [`call`](mod@call) for Vimscript
//! reaching Lua, [`excmds`] and [`ucmd`] for the `:` commands, [`callable`]
//! for a Lua value Vimscript can call, [`schedule`](mod@schedule) and [`luv`] for the event
//! loop, [`print`] for the output, [`error`] for what a failure becomes, and
//! [`expand`] for command-line completion.
//!
//! The editor-wide cells every one of those children reads live here; the
//! states themselves are [`state`]'s, which lends them out as a [`Stack`].
//!
//! [`Stack`]: crate::lua::stack::Stack

// No `forbid(unsafe_code)`: it would reach `luv` and `state`, the two
// children that stay on the perimeter (luv's C hooks, and making and closing
// states). Everything else here forbids unsafe code on its own.
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use core::ffi::c_int;

use crate::global_cell::{GlobalCell, SharedCell};
use crate::lua::ffi::LUA_REFNIL;
use crate::types::{LuaRef, LuaRetMode, uint64_t};

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

pub(crate) use self::call::*;
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

// -- The editor-wide state --------------------------------------------------

/// How deep inside a *fast* callback we are: a libuv callback running on the
/// main loop, where nothing that touches the editor's own state may run.
/// Counted rather than flagged, because one may nest inside another.
static in_fast_callback: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether this process is `nvim -l`, with no editor at all: errors go to
/// stderr and the runtime path is never rebuilt.
static in_script: SharedCell<bool> = SharedCell::new(false);

/// The stock `require`, kept so `--startuptime`'s wrapper can delegate to it.
static require_ref: GlobalCell<LuaRef> = GlobalCell::new(LUA_REFNIL);
