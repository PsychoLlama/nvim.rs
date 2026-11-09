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

use core::ffi::c_int;

use crate::global_cell::{GlobalCell, SharedCell};
use crate::lua::ffi::LUA_REFNIL;
use crate::os::libc::uint64_t;
use crate::types::LuaRef;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuaRetMode {
    /// The first result, as an `Object`.
    Object,
    /// The first result read as a boolean: nil or `false` is false.
    NilBool,
    /// The first result kept in the registry: a `LuaRef` to it.
    LuaRef,
    /// Every result, as an `Array`.
    Multi,
}

/// `call_func`'s answers: the two this module can produce.
pub type FuncErrorType = ::core::ffi::c_uint;

/// The channel id an api call made from inside the editor carries.
const INTERNAL_CALL_MASK: uint64_t = 1 << (uint64_t::BITS - 1);
pub const VIML_INTERNAL_CALL: uint64_t = INTERNAL_CALL_MASK;
pub const LUA_INTERNAL_CALL: uint64_t = VIML_INTERNAL_CALL + 1;

// -- The editor-wide state --------------------------------------------------

/// How deep inside a *fast* callback we are: a libuv callback running on the
/// main loop, where nothing that touches the editor's own state may run.
/// Counted rather than flagged, because one may nest inside another.
static IN_FAST_CALLBACK: GlobalCell<c_int> = GlobalCell::new(0);

/// Whether this process is `nvim -l`, with no editor at all: errors go to
/// stderr and the runtime path is never rebuilt.
static IN_SCRIPT: SharedCell<bool> = SharedCell::new(false);

/// The stock `require`, kept so `--startuptime`'s wrapper can delegate to it.
static REQUIRE_REF: GlobalCell<LuaRef> = GlobalCell::new(LUA_REFNIL);

#[derive(Debug)]
pub struct NluaRefState {
    pub nil_ref: LuaRef,
    pub empty_dict_ref: LuaRef,
    pub ref_count: ::core::ffi::c_int,
}
