//! References into a table of the caller's own: `luaL_ref`/`luaL_unref`
//! against any table, not only the registry, and uncounted.
//!
//! [`Stack::reference`] is the editor's: registry entries it counts for
//! its leak check. A Lua C module that parks values in a private table
//! (`vim.mpack`'s codecs, between two calls of a suspended walk) needs the
//! same free-list allocator over that table, on any state.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `lua/stack/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use super::{At, Stack};
use crate::lua::ffi::{luaL_ref, luaL_unref};

impl Stack {
    /// Pop the value on top into the table at `table` and answer its
    /// reference there: `luaL_ref(L, table)` (`LUA_REFNIL` for a nil, which
    /// stores nothing).
    pub fn reference_in(&mut self, table: At) -> c_int {
        debug_assert!(self.depth() >= 1, "a view references a value it pushed");
        unsafe { luaL_ref(self.l(), table.0) }
    }

    /// Release `reference` in the table at `table`: `luaL_unref`. A
    /// sentinel (`LUA_NOREF`, `LUA_REFNIL`) is a no-op.
    pub fn unreference_in(&mut self, table: At, reference: c_int) {
        unsafe { luaL_unref(self.l(), table.0, reference) };
    }
}
