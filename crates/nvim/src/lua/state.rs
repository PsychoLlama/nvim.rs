//! What the Lua state holds on the editor's behalf.
//!
//! `nlua_global_refs` is the registry of `LuaRef`s the editor handed out and
//! must eventually unref; the rest is the runtime's own bookkeeping --
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

use crate::global_cell::{GlobalCell, SharedCell};
use crate::types::{LuaRetMode, NluaRefState, uint64_t};
use core::ffi::c_int;

pub(crate) const kRetObject: LuaRetMode = 0;
pub(crate) const LUA_GLOBALSINDEX: c_int = -10002 as c_int;
pub(crate) static nlua_global_refs: GlobalCell<*mut NluaRefState> =
    GlobalCell::new(::core::ptr::null_mut::<NluaRefState>());
pub(crate) static nlua_disable_preload: SharedCell<bool> = SharedCell::new(false);
pub(crate) static tslua_query_parse_count: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
