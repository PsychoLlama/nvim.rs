//! What the Lua state holds on the editor's behalf.
//!
//! [`MAIN_REFS`] counts the `LuaRef`s the editor handed out against the main
//! state and must eventually unref, and holds its two sentinels; the rest is the runtime's own bookkeeping --
//! whether `package.preload` was suppressed, how many treesitter queries
//! have been parsed, and the two constants the FFI bindings index by.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

use crate::global_cell::{GlobalCell, SharedCell};
use crate::lua::executor::{LuaRetMode, NluaRefState};
use crate::lua::ffi::LUA_NOREF;
use crate::os::libc::uint64_t;

pub(crate) const RET_OBJECT: LuaRetMode = 0;
/// The main state's reference bookkeeping: how many references it holds
/// (what the leak checks read), and the `vim.NIL` and `vim.empty_dict()`
/// sentinels, which are references too.
pub(crate) static MAIN_REFS: GlobalCell<NluaRefState> = GlobalCell::new(NluaRefState {
    nil_ref: LUA_NOREF,
    empty_dict_ref: LUA_NOREF,
    ref_count: 0,
});
pub(crate) static NLUA_DISABLE_PRELOAD: SharedCell<bool> = SharedCell::new(false);
pub(crate) static TSLUA_QUERY_PARSE_COUNT: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
