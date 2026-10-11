#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use core::ffi::CStr;
use std::rc::Rc;

/// One user command, as `:command` or `nvim_create_user_command()`
/// defined it.
///
/// Neither `Copy` nor `Clone`: an entry owns its three Lua references, and
/// dropping it gives the references back. A table entry is therefore
/// moved out of its table before it is dropped -- giving a reference back
/// calls into Lua, which must not happen with the table borrowed. The
/// strings are shared (`Rc`), so a copy of what a run or a listing needs
/// costs a count, not an allocation.
pub struct UserCmd {
    pub name: Rc<CStr>,
    pub argt: ExArgt,
    /// The definition, `<...>` codes unexpanded and key codes already
    /// replaced; a Lua command keeps its description here.
    pub replacement: Rc<CStr>,
    /// `-count=N`/`-range=N`'s default, or -1.
    pub default_count: int64_t,
    pub complete: ExpandContext,
    pub addr_type: CmdAddr,
    pub script_ctx: ScriptCtx,
    /// `-complete=custom,{func}`'s function.
    pub complete_arg: Option<Rc<CStr>>,
    pub complete_luaref: LuaRef,
    pub preview_luaref: LuaRef,
    /// The Lua function a Lua command runs, or `LUA_NOREF`.
    pub luaref: LuaRef,
}

impl Drop for UserCmd {
    fn drop(&mut self) {
        for reference in [self.complete_luaref, self.preview_luaref, self.luaref] {
            // A sentinel is a no-op there.
            crate::lua::executor::release_luaref(reference);
        }
    }
}
